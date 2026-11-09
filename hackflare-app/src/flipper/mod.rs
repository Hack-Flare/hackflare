//! Database-backed feature flags grouped by tag.
//!
//! Flags are disabled by default.
//! A feature becomes toggleable in the admin dashboard after it is registered
//! Stays disabled until enabled

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query_as};
use tokio::sync::{Mutex, RwLock};
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
    cache: Arc<RwLock<HashMap<String, FeatureFlag>>>,
    cache_update: Arc<Mutex<()>>,
}

impl FeatureFlags {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_update: Arc::new(Mutex::new(())),
        }
    }

    /// Load flags into memory and refresh the cache every 30 seconds.
    pub async fn initialize(&self) -> Result<()> {
        self.refresh().await?;
        let flags = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Err(error) = flags.refresh().await {
                    warn!(%error, "failed to refresh feature flag cache");
                }
            }
        });
        Ok(())
    }

    /// Replace the in-memory catalogue with the current database values.
    pub async fn refresh(&self) -> Result<()> {
        let _update = self.cache_update.lock().await;
        let rows = query_as::<_, FeatureFlag>(
            r#"
            SELECT id, key, tag, description, enabled, updated_by, created_at, updated_at
            FROM feature_flags
            ORDER BY tag, key
            "#,
        )
        .fetch_all(&self.db)
        .await?;
        let flags = rows
            .into_iter()
            .map(|flag| (flag.key.clone(), flag))
            .collect();
        *self.cache.write().await = flags;
        Ok(())
    }

    /// Every registered flag, grouped by tag when read in order.
    pub async fn list(&self) -> Result<Vec<FeatureFlag>> {
        let mut flags: Vec<_> = self.cache.read().await.values().cloned().collect();
        flags.sort_by(|left, right| (&left.tag, &left.key).cmp(&(&right.tag, &right.key)));
        Ok(flags)
    }

    pub async fn count(&self) -> Result<i64> {
        Ok(self.cache.read().await.len() as i64)
    }

    pub async fn find(&self, key: &str) -> Result<Option<FeatureFlag>> {
        Ok(self.cache.read().await.get(key).cloned())
    }

    /// Whether a feature is enabled. Unknown flags are always disabled.
    pub async fn is_enabled(&self, key: &str) -> Result<bool> {
        Ok(self
            .cache
            .read()
            .await
            .get(key)
            .is_some_and(|flag| flag.enabled))
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

        let _update = self.cache_update.lock().await;
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
        self.cache
            .write()
            .await
            .insert(flag.key.clone(), flag.clone());
        Ok(flag)
    }

    /// Enable or disable a flag. Returns `None` when the flag is not registered.
    pub async fn set_enabled(
        &self,
        key: &str,
        enabled: bool,
        updated_by: Option<&str>,
    ) -> Result<Option<FeatureFlag>> {
        let _update = self.cache_update.lock().await;
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
        if let Some(flag) = &flag {
            self.cache
                .write()
                .await
                .insert(flag.key.clone(), flag.clone());
        }
        Ok(flag)
    }
}
