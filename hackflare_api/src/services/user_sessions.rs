use std::net::IpAddr;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres, Row, query, query_as};
use uuid::Uuid;

use crate::models::db::UserSession;

#[derive(Clone)]
pub(crate) struct UserSessionsService {
    db: PgPool,
}

impl UserSessionsService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn create_with<'e, E>(
        executor: E,
        user_id: &str,
        ip_address: IpAddr,
        expires_at: DateTime<Utc>,
    ) -> Result<Uuid>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let row = query(
            r#"
            INSERT INTO user_sessions (user_id, ip_address, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(ip_address)
        .bind(expires_at)
        .fetch_one(executor)
        .await?;

        let id: Uuid = row.try_get("id")?;

        Ok(id)
    }

    pub(crate) async fn revoke(&self, id: &Uuid) -> Result<bool> {
        let rows = query(
            r#"
            UPDATE user_sessions
            SET revoked_at = NOW()
            WHERE id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    pub(crate) async fn get_by_id(&self, id: &Uuid) -> Result<Option<UserSession>> {
        let session = query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, ip_address, expires_at, created_at, revoked_at
            FROM user_sessions
            WHERE id = $1
            AND revoked_at IS NULL
            AND expires_at >= NOW()
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(session)
    }

    pub(crate) async fn revoke_all_for_user(&self, user_id: &str) -> Result<bool> {
        let rows = query(
            r#"
            UPDATE user_sessions
            SET revoked_at = NOW()
            WHERE user_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    pub(crate) async fn get_all_for_user(
        &self,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserSession>> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);

        let sessions = query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, ip_address, expires_at, created_at, revoked_at
            FROM user_sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(sessions)
    }
}
