use std::net::IpAddr;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres, query, query_as};
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
        let id = query!(
            r#"
            INSERT INTO user_sessions (user_id, ip_address, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            user_id,
            ip_address as _,
            expires_at,
        )
        .fetch_one(executor)
        .await?
        .id;

        Ok(id)
    }

    pub(crate) async fn get_by_id(&self, id: &Uuid) -> Result<Option<UserSession>> {
        let session = query_as!(
            UserSession,
            r#"
            SELECT id, user_id, ip_address as "ip_address: IpAddr", expires_at, created_at, revoked_at
            FROM user_sessions
            WHERE id = $1
            AND revoked_at IS NULL
            AND expires_at >= NOW()
            LIMIT 1
            "#,
            id,
        ).fetch_optional(&self.db).await?;

        Ok(session)
    }

    pub(crate) async fn get_all_for_user(&self, user_id: &str) -> Result<Vec<UserSession>> {
        let sessions = query_as!(
            UserSession,
            r#"
            SELECT id, user_id, ip_address as "ip_address: IpAddr", expires_at, created_at, revoked_at
            FROM user_sessions
            WHERE user_id = $1
            "#,
            user_id,
        ).fetch_all(&self.db).await?;

        // TODO: implement pagination
        if sessions.len() > 100 {
            warn!("user sessions count exceeds 100, pagination strongly recommended")
        }

        Ok(sessions)
    }
}
