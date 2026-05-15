use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::extract::FromRef;
use sqlx::{
    PgPool,
    migrate::{Migrate, Migrator},
    postgres::PgPoolOptions,
};

use crate::{
    config::{Config, Environment},
    services::users::UsersService,
};

static MIGRATOR: Migrator = sqlx::migrate!("../migrations");

#[derive(Clone, FromRef)]
pub struct AppState {
    pub(crate) config: Arc<Config>,
    pub(crate) http_client: reqwest::Client,
    pub(crate) db: PgPool,

    // -- services --
    pub(crate) users: UsersService,
}

#[instrument(skip(db))]
async fn migrate_or_verify(db: &PgPool, env: &Environment) -> Result<()> {
    if env.is_prod() {
        MIGRATOR.run(db).await?;
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

        for migration in MIGRATOR.iter() {
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
            > MIGRATOR
                .migrations
                .iter()
                .filter(|m| !m.migration_type.is_down_migration())
                .count()
        {
            for applied_version in applied_map.keys() {
                if !MIGRATOR
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

        migrate_or_verify(&db, &config.environment).await?;

        let users = UsersService::new(db.clone());

        Ok(Self {
            config: Arc::new(config),
            http_client,
            db,
            users,
        })
    }
}
