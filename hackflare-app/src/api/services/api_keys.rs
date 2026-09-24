use anyhow::Result;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, query, query_as};
use uuid::Uuid;

const KEY_PREFIX: &str = "hf_v1";
const KEY_ID_LEN: usize = 16;
const SECRET_LEN: usize = 64;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub(crate) struct ApiKey {
    pub(crate) id: Uuid,
    pub(crate) user_id: String,
    pub(crate) name: String,
    pub(crate) prefix: String,
    pub(crate) key_id: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) last_used_at: Option<DateTime<Utc>>,
    pub(crate) revoked_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub(crate) struct ApiKeysService {
    db: PgPool,
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn hash_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

fn hash_secret(secret: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_key_id() -> String {
    let bytes: [u8; 8] = rand::random();
    hex::encode(bytes)
}

fn generate_secret() -> String {
    let bytes: [u8; SECRET_LEN / 2] = rand::random();
    hex::encode(bytes)
}

fn generate_raw_key() -> String {
    let key_id = generate_key_id();
    let secret = generate_secret();
    format!("{KEY_PREFIX}_{key_id}_{secret}")
}

fn parse_v1_key(raw: &str) -> Option<(&str, &str)> {
    let rest = raw.strip_prefix("hf_v1_")?;
    let parts: Vec<&str> = rest.splitn(2, '_').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

impl ApiKeysService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn create(&self, user_id: &str, name: &str) -> Result<(ApiKey, String)> {
        let raw = generate_raw_key();
        let (key_id, secret) = parse_v1_key(&raw).expect("generated key should be valid v1 format");
        let hash = hash_secret(secret, key_id);
        let prefix = raw[..KEY_PREFIX.len() + 1 + KEY_ID_LEN].to_string();

        let row = query(
            r#"
            INSERT INTO api_keys (user_id, name, key_hash, prefix, key_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, prefix, key_id, created_at, last_used_at, revoked_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(&hash)
        .bind(&prefix)
        .bind(key_id)
        .fetch_one(&self.db)
        .await?;

        let key = ApiKey {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            prefix: row.try_get("prefix")?,
            key_id: row.try_get("key_id")?,
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at")?,
            revoked_at: row.try_get("revoked_at")?,
        };

        Ok((key, raw))
    }

    pub(crate) async fn list(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let keys = query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, prefix, key_id, created_at, last_used_at, revoked_at
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
        if let Some((key_id, secret)) = parse_v1_key(raw) {
            let row = query(
                r#"
                SELECT id, user_id, name, prefix, key_id, key_hash, created_at, last_used_at, revoked_at
                FROM api_keys
                WHERE key_id = $1 AND revoked_at IS NULL
                "#,
            )
            .bind(key_id)
            .fetch_optional(&self.db)
            .await?;

            if let Some(row) = row {
                let stored_hash: String = row.try_get("key_hash")?;
                if constant_time_eq(&hash_secret(secret, key_id), &stored_hash) {
                    return Ok(Some(ApiKey {
                        id: row.try_get("id")?,
                        user_id: row.try_get("user_id")?,
                        name: row.try_get("name")?,
                        prefix: row.try_get("prefix")?,
                        key_id: row.try_get("key_id")?,
                        created_at: row.try_get("created_at")?,
                        last_used_at: row.try_get("last_used_at")?,
                        revoked_at: row.try_get("revoked_at")?,
                    }));
                }
            }
            return Ok(None);
        }

        let hash = hash_key(raw);
        let key = query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, prefix, key_id, created_at, last_used_at, revoked_at
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
