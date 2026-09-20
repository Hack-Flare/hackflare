use anyhow::Result;
use chrono::Utc;
use sqlx::{PgPool, query, query_as};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct PasswordResetService {
    db: PgPool,
}

#[derive(sqlx::FromRow)]
struct ResetTokenRow {
    #[allow(dead_code)]
    id: Uuid,
    user_id: String,
}

impl PasswordResetService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn create_token(&self, user_id: &str) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        query(
            r#"
            INSERT INTO password_reset_tokens (user_id, token, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.db)
        .await?;

        Ok(token)
    }

    pub(crate) async fn consume_token(&self, token: &str) -> Result<Option<String>> {
        let row: Option<ResetTokenRow> = query_as(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE token = $1
              AND expires_at > NOW()
              AND used_at IS NULL
            RETURNING id, user_id
            "#,
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| r.user_id))
    }

    pub(crate) async fn revoke_all_for_user(&self, user_id: &str) -> Result<()> {
        query(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE user_id = $1 AND used_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
