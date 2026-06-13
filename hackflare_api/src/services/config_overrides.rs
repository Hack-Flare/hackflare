use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, query, query_as};

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
    pub(crate) default_value: Option<String>,
    pub(crate) default_override: bool,
    pub(crate) editable: bool,
    pub(crate) requires_restart: bool,
    pub(crate) updated_at: Option<DateTime<Utc>>,
    pub(crate) updated_by: Option<String>,
}

static CONFIG_METADATA: &[ConfigMeta] = &[
    ConfigMeta {
        // The env variable
        key: "API_BIND_ADDR",
        // Label for UI display
        label: "Bind Address",
        // Keep description concise since it's shown in a table with limited space
        description: "HTTP server bind address",
        // Default value if not set in env and no override exists
        default_value: Some("0.0.0.0:8080"),
        // If true then takes priority over env var
        default_override: false,
        // If it can be edited from the admin UI
        editable: true,
        // If it requires a Restart to take effect (for UI warning)
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_DNS_BIND_ADDR",
        label: "DNS Bind Address",
        description: "DNS server bind address",
        default_value: Some("0.0.0.0:5353"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_ENVIRONMENT",
        label: "Environment",
        description: "Production or development mode",
        default_value: Some("production"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_AUTO_MIGRATE",
        label: "Auto Migrate",
        description: "Run database migrations on startup",
        default_value: Some("true"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_HCA_CLIENT_ID",
        label: "HCA Client ID",
        description: "Hack Club Auth client ID",
        default_value: None,
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_HCA_CLIENT_SECRET",
        label: "HCA Client Secret",
        description: "Hack Club Auth client secret",
        default_value: None,
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_HCA_REDIRECT_URI",
        label: "HCA Redirect URI",
        description: "OAuth callback URL",
        default_value: None,
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_ACCESS_TOKEN_MINUTES",
        label: "Access Token TTL",
        description: "Access token lifetime in minutes",
        default_value: Some("15"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_REFRESH_TOKEN_DAYS",
        label: "Refresh Token TTL",
        description: "Refresh token lifetime in days",
        default_value: Some("30"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_SESSION_INACTIVITY_MINUTES",
        label: "Session Inactivity",
        description: "Session timeout on inactivity",
        default_value: Some("15"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_DNS_NAMESERVERS",
        label: "DNS Nameservers",
        description: "Comma-separated expected nameservers",
        default_value: Some("ns1.hackflare.dev,ns2.hackflare.dev"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_CLIENT_IP_SOURCE",
        label: "Client IP Source",
        description: "How to determine client IP",
        default_value: Some("ConnectInfo"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_ADMIN_EMAILS",
        label: "Admin Emails",
        description: "Comma-separated list of admin email addresses",
        default_value: Some(""),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "SLACK_WEBHOOK_URL",
        label: "Slack Webhook URL",
        description: "Incoming webhook for contact form",
        default_value: None,
        default_override: true,
        editable: true,
        requires_restart: false,
    },
    ConfigMeta {
        key: "DATABASE_URL",
        label: "Database URL",
        description: "PostgreSQL connection string",
        default_value: Some("postgres://hackflare:hackflare@db:5432/hackflare"),
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "API_JWT_SECRET",
        label: "JWT Secret",
        description: "Base64-encoded JWT signing secret",
        default_value: None,
        default_override: false,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "SMTP_HOST",
        label: "SMTP Host",
        description: "SMTP server hostname",
        default_value: None,
        default_override: true,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "SMTP_PORT",
        label: "SMTP Port",
        description: "SMTP server port",
        default_value: Some("587"),
        default_override: true,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "SMTP_USERNAME",
        label: "SMTP Username",
        description: "SMTP authentication username",
        default_value: None,
        default_override: true,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "SMTP_PASSWORD",
        label: "SMTP Password",
        description: "SMTP authentication password",
        default_value: None,
        default_override: true,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "SMTP_FROM",
        label: "SMTP From",
        description: "Sender email address for outgoing mail",
        default_value: None,
        default_override: true,
        editable: true,
        requires_restart: true,
    },
    ConfigMeta {
        key: "FRONTEND_URL",
        label: "Frontend URL",
        description: "Public URL of the frontend (for password reset links)",
        default_value: Some(""),
        default_override: true,
        editable: true,
        requires_restart: true,
    },
];

pub(crate) struct ConfigMeta {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) description: &'static str,
    pub(crate) default_value: Option<&'static str>,
    pub(crate) default_override: bool,
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
        CONFIG_METADATA.iter().any(|m| m.key == key && m.editable)
    }
}
