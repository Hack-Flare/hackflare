use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query, query_as};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub(crate) struct ConfigOverride {
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) updated_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConfigEntry {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) env_value: Option<String>,
    pub(crate) override_value: Option<String>,
    pub(crate) effective_value: String,
    pub(crate) editable: bool,
    pub(crate) requires_restart: bool,
    pub(crate) updated_at: Option<DateTime<Utc>>,
    pub(crate) updated_by: Option<String>,
}

static CONFIG_METADATA: &[ConfigMeta] = &[
    ConfigMeta { key: "API_BIND_ADDR", label: "Bind Address", description: "HTTP server bind address", editable: true, requires_restart: true },
    ConfigMeta { key: "API_DNS_BIND_ADDR", label: "DNS Bind Address", description: "DNS server bind address", editable: true, requires_restart: true },
    ConfigMeta { key: "API_ENVIRONMENT", label: "Environment", description: "Production or development mode", editable: true, requires_restart: true },
    ConfigMeta { key: "API_AUTO_MIGRATE", label: "Auto Migrate", description: "Run database migrations on startup", editable: true, requires_restart: true },
    ConfigMeta { key: "API_HCA_CLIENT_ID", label: "HCA Client ID", description: "Hack Club Auth client ID", editable: true, requires_restart: true },
    ConfigMeta { key: "API_HCA_CLIENT_SECRET", label: "HCA Client Secret", description: "Hack Club Auth client secret", editable: true, requires_restart: true },
    ConfigMeta { key: "API_HCA_REDIRECT_URI", label: "HCA Redirect URI", description: "OAuth callback URL", editable: true, requires_restart: true },
    ConfigMeta { key: "API_ACCESS_TOKEN_MINUTES", label: "Access Token TTL", description: "Access token lifetime in minutes", editable: true, requires_restart: true },
    ConfigMeta { key: "API_REFRESH_TOKEN_DAYS", label: "Refresh Token TTL", description: "Refresh token lifetime in days", editable: true, requires_restart: true },
    ConfigMeta { key: "API_SESSION_INACTIVITY_MINUTES", label: "Session Inactivity", description: "Session timeout on inactivity", editable: true, requires_restart: true },
    ConfigMeta { key: "API_DNS_NAMESERVERS", label: "DNS Nameservers", description: "Comma-separated expected nameservers", editable: true, requires_restart: true },
    ConfigMeta { key: "API_CLIENT_IP_SOURCE", label: "Client IP Source", description: "How to determine client IP", editable: true, requires_restart: true },
    ConfigMeta { key: "API_ADMIN_EMAILS", label: "Admin Emails", description: "Comma-separated list of admin email addresses", editable: true, requires_restart: true },
    ConfigMeta { key: "SLACK_WEBHOOK_URL", label: "Slack Webhook URL", description: "Incoming webhook for contact form", editable: true, requires_restart: false },
    ConfigMeta { key: "DATABASE_URL", label: "Database URL", description: "PostgreSQL connection string", editable: false, requires_restart: true },
    ConfigMeta { key: "API_JWT_SECRET", label: "JWT Secret", description: "Base64-encoded JWT signing secret", editable: false, requires_restart: true },
];

pub(crate) struct ConfigMeta {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) description: &'static str,
    pub(crate) editable: bool,
    pub(crate) requires_restart: bool,
}

#[derive(Clone)]
pub(crate) struct ConfigOverridesService {
    db: PgPool,
}

impl ConfigOverridesService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn list_overrides(&self) -> Result<Vec<ConfigOverride>> {
        let overrides = query_as::<_, ConfigOverride>(
            "SELECT key, value, updated_at, updated_by FROM config_overrides ORDER BY key",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(overrides)
    }

    pub(crate) async fn get_override(&self, key: &str) -> Result<Option<ConfigOverride>> {
        let ov = query_as::<_, ConfigOverride>(
            "SELECT key, value, updated_at, updated_by FROM config_overrides WHERE key = $1",
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await?;
        Ok(ov)
    }

    pub(crate) async fn upsert(&self, key: &str, value: &str, updated_by: &str) -> Result<()> {
        query(
            r#"
            INSERT INTO config_overrides (key, value, updated_by, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_by = EXCLUDED.updated_by,
                updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(updated_by)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub(crate) async fn delete(&self, key: &str) -> Result<bool> {
        let rows = query("DELETE FROM config_overrides WHERE key = $1")
            .bind(key)
            .execute(&self.db)
            .await?;
        Ok(rows.rows_affected() > 0)
    }

    pub(crate) fn get_known_keys() -> &'static [ConfigMeta] {
        CONFIG_METADATA
    }

    pub(crate) fn is_editable(key: &str) -> bool {
        CONFIG_METADATA
            .iter()
            .any(|m| m.key == key && m.editable)
    }
}
