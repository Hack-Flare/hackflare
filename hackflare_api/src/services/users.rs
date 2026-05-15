use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query, query_as};

use crate::models::{HcaUser, db::User};

#[derive(Clone)]
pub(crate) struct UsersService {
    db: PgPool,
}

impl UsersService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn upsert(
        &self,
        user: &HcaUser,
        access_token: &str,
        refresh_token: &str,
        token_expires_at: DateTime<Utc>,
    ) -> Result<()> {
        query!(
            r#"
            INSERT INTO users (id, email, slack_id, first_name, last_name, verification_status, ysws_eligible, hca_access_token, hca_refresh_token, hca_token_expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                email = EXCLUDED.email,
                slack_id = EXCLUDED.slack_id,
                first_name = EXCLUDED.first_name,
                last_name = EXCLUDED.last_name,
                verification_status = EXCLUDED.verification_status,
                ysws_eligible = EXCLUDED.ysws_eligible,
                updated_at = NOW(),
                hca_access_token = EXCLUDED.hca_access_token,
                hca_refresh_token = EXCLUDED.hca_refresh_token,
                hca_token_expires_at = EXCLUDED.hca_token_expires_at;
            "#,
            user.id,
            user.primary_email,
            user.slack_id,
            user.first_name,
            user.last_name,
            user.verification_status,
            user.ysws_eligible,
            access_token,
            refresh_token,
            token_expires_at,
        ).execute(&self.db).await?;

        Ok(())
    }

    pub(crate) async fn get_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = query_as!(User, "SELECT * from users WHERE id = $1 LIMIT 1", id)
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }
}
