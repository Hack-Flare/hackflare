use anyhow::Result;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, query, query_as};
use uuid::Uuid;

const KEY_PREFIX: &str = "hf";
const PREFIX_LEN: usize = 8;
const SECRET_LEN: usize = 32;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub(crate) struct ApiKey {
    pub(crate) id: Uuid,
    pub(crate) user_id: String,
    pub(crate) name: String,
    pub(crate) prefix: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) last_used_at: Option<DateTime<Utc>>,
    pub(crate) revoked_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub(crate) struct ApiKeysService {
    db: PgPool,
}

fn hash_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_raw_key() -> String {
    let prefix = &Uuid::new_v4().to_string()[..PREFIX_LEN];
    let secret =
        Uuid::new_v4().to_string().replace('-', "") + &Uuid::new_v4().to_string().replace('-', "");
    let secret = &secret[..SECRET_LEN];
    format!("{KEY_PREFIX}_{prefix}_{secret}")
}

impl ApiKeysService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn create(&self, user_id: &str, name: &str) -> Result<(ApiKey, String)> {
        let raw = generate_raw_key();
        let hash = hash_key(&raw);
        let prefix = raw[..KEY_PREFIX.len() + 1 + PREFIX_LEN].to_string();

        let row = query(
            r#"
            INSERT INTO api_keys (user_id, name, key_hash, prefix)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, name, prefix, created_at, last_used_at, revoked_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(&hash)
        .bind(&prefix)
        .fetch_one(&self.db)
        .await?;

        let key = ApiKey {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            prefix: row.try_get("prefix")?,
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at")?,
            revoked_at: row.try_get("revoked_at")?,
        };

        Ok((key, raw))
    }

    pub(crate) async fn list(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let keys = query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, prefix, created_at, last_used_at, revoked_at
            FROM api_keys
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;
        Ok(keys)
    }

    pub(crate) async fn revoke(&self, id: Uuid, user_id: &str) -> Result<bool> {
        let rows = query(
            r#"
            UPDATE api_keys
            SET revoked_at = NOW()
            WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.db)
        .await?;
        Ok(rows.rows_affected() > 0)
    }

    pub(crate) async fn find_by_key(&self, raw: &str) -> Result<Option<ApiKey>> {
        let hash = hash_key(raw);
        let key = query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, prefix, created_at, last_used_at, revoked_at
            FROM api_keys
            WHERE key_hash = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(&hash)
        .fetch_optional(&self.db)
        .await?;
        Ok(key)
    }

    pub(crate) async fn update_last_used(&self, id: Uuid) -> Result<()> {
        query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}
