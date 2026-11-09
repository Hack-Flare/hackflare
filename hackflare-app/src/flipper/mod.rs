//! Database-backed feature flags grouped by tag.
//!
//! Flags are disabled by default.
//! A feature becomes toggleable in the admin dashboard after it is registered
//! Stays disabled until enabled

use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query_as, query_scalar};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeatureFlag {
    pub id: Uuid,
    pub key: String,
    pub tag: String,
    pub description: String,
    pub enabled: bool,
    pub updated_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FeatureFlags {
    db: PgPool,
}

impl FeatureFlags {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Every registered flag, grouped by tag when read in order.
    pub async fn list(&self) -> Result<Vec<FeatureFlag>> {
        let flags = query_as::<_, FeatureFlag>(
            r#"
            SELECT id, key, tag, description, enabled, updated_by, created_at, updated_at
            FROM feature_flags
            ORDER BY tag, key
            "#,
        )
        .fetch_all(&self.db)
        .await?;
        Ok(flags)
    }

    pub async fn count(&self) -> Result<i64> {
        let count = query_scalar("SELECT COUNT(*) FROM feature_flags")
            .fetch_one(&self.db)
            .await?;
        Ok(count)
    }

    pub async fn find(&self, key: &str) -> Result<Option<FeatureFlag>> {
        let flag = query_as::<_, FeatureFlag>(
            r#"
            SELECT id, key, tag, description, enabled, updated_by, created_at, updated_at
            FROM feature_flags
            WHERE key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await?;
        Ok(flag)
    }

    /// Whether a feature is enabled. Unknown flags are always disabled.
    pub async fn is_enabled(&self, key: &str) -> Result<bool> {
        let enabled = query_scalar::<_, bool>(
            r#"
            SELECT enabled
            FROM feature_flags
            WHERE key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await?;
        Ok(enabled.unwrap_or(false))
    }

    /// Add a flag to the catalogue or refresh its metadata without changing its current enabled state.
    pub async fn register(&self, key: &str, tag: &str, description: &str) -> Result<FeatureFlag> {
        let key = key.trim();
        let tag = tag.trim();
        if key.is_empty() {
            bail!("feature flag key is required");
        }
        if tag.is_empty() {
            bail!("feature flag tag is required");
        }

        let flag = query_as::<_, FeatureFlag>(
            r#"
            INSERT INTO feature_flags (key, tag, description)
            VALUES ($1, $2, $3)
            ON CONFLICT (key) DO UPDATE
            SET tag = EXCLUDED.tag, description = EXCLUDED.description
            RETURNING id, key, tag, description, enabled, updated_by, created_at, updated_at
            "#,
        )
        .bind(key)
        .bind(tag)
        .bind(description)
        .fetch_one(&self.db)
        .await?;
        Ok(flag)
    }

    /// Enable or disable a flag. Returns `None` when the flag is not registered.
    pub async fn set_enabled(
        &self,
        key: &str,
        enabled: bool,
        updated_by: Option<&str>,
    ) -> Result<Option<FeatureFlag>> {
        let flag = query_as::<_, FeatureFlag>(
            r#"
            UPDATE feature_flags
            SET enabled = $2, updated_by = $3, updated_at = NOW()
            WHERE key = $1
            RETURNING id, key, tag, description, enabled, updated_by, created_at, updated_at
            "#,
        )
        .bind(key)
        .bind(enabled)
        .bind(updated_by)
        .fetch_optional(&self.db)
        .await?;
        Ok(flag)
    }
}
