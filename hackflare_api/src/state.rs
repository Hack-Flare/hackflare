use std::{path::Path, sync::Arc, time::Duration};

use anyhow::Result;
use axum::extract::FromRef;
use hackflare_dns::{
    DnsConfig,
    ns::{AuthorityStore, PostgresPersistence, ZonePersistence},
};
use sqlx::{
    PgPool,
    migrate::{Migrate, Migrator},
    postgres::PgPoolOptions,
};

use crate::{
    config::Config,
    services::{
        api_keys::ApiKeysService, config_overrides::ConfigOverridesService,
        email::EmailService, user_sessions::UserSessionsService, users::UsersService,
    },
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<Config>,
    pub(crate) http_client: reqwest::Client,
    pub db: PgPool,

    // -- dns --
    pub dns_authority: Arc<AuthorityStore>,

    // -- services --
    pub(crate) api_keys: ApiKeysService,
    pub(crate) users: UsersService,
    pub(crate) user_sessions: UserSessionsService,
    pub(crate) config_overrides: ConfigOverridesService,
    pub(crate) email: Option<EmailService>,
}

#[instrument(skip(db))]
async fn migrate_or_verify(db: &PgPool, config: &Config) -> Result<()> {
    let migrations_path =
        std::env::var("MIGRATIONS_PATH").unwrap_or_else(|_| "../database/migrations".to_string());
    let migrator = Migrator::new(Path::new(&migrations_path)).await?;

    if config.auto_migrate {
        migrator.run(db).await?;
    } else {
        let mut conn = db.acquire().await?;

        if let Some(version) = conn.dirty_version().await? {
            anyhow::bail!(
                "database in dirty state at version {}, manual intervention required",
                version
            );
        }

        let applied_map: std::collections::HashMap<_, _> = conn
            .list_applied_migrations()
            .await?
            .into_iter()
            .map(|m| (m.version, m.checksum))
            .collect();

        debug!("number of applied migrations: {}", applied_map.len());

        for migration in migrator.iter() {
            if migration.migration_type.is_down_migration() {
                continue;
            }

            match applied_map.get(&migration.version) {
                Some(applied) => {
                    if migration.checksum.as_ref() != applied.as_ref() {
                        anyhow::bail!(
                            "checksum mismatch for migration {} '{}', manual intervention required",
                            migration.version,
                            migration.description
                        )
                    }
                }
                None => {
                    anyhow::bail!("migration {} missing from db", migration.version)
                }
            }
        }

        if applied_map.len()
            > migrator
                .migrations
                .iter()
                .filter(|m| !m.migration_type.is_down_migration())
                .count()
        {
            for applied_version in applied_map.keys() {
                if !migrator
                    .migrations
                    .iter()
                    .any(|m| m.version == *applied_version)
                {
                    anyhow::bail!(
                        "Database has migration {} which is missing locally.",
                        applied_version
                    );
                }
            }
        }

        info!("database schema verified");
    }
    Ok(())
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        info!(%config.environment, "setting up app state");

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to create http client");
        info!("http client initialized");

        let db = PgPoolOptions::new()
            .max_connections(50)
            .connect(config.database_url.as_str())
            .await?;
        info!("database connection pool initialized");

        migrate_or_verify(&db, &config).await?;

        let users = UsersService::new(db.clone());
        let user_sessions = UserSessionsService::new(db.clone());
        let config_overrides = ConfigOverridesService::new(db.clone());
        let api_keys = ApiKeysService::new(db.clone());

        let persistence: Arc<dyn ZonePersistence> = Arc::new(PostgresPersistence::new(db.clone()));
        let dns_authority = Arc::new(AuthorityStore::with_persistence(
            DnsConfig::from_env(),
            persistence,
        ));
        dns_authority.load_zones_from_storage().await?;
        info!("dns zones loaded from storage");

        let email = config.smtp.as_ref().map(EmailService::new);

        Ok(Self {
            config: Arc::new(config),
            http_client,
            db,
            dns_authority,
            api_keys,
            users,
            user_sessions,
            config_overrides,
            email,
        })
    }
}
