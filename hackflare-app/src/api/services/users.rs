use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Transaction, query, query_as};
use uuid::Uuid;

use crate::api::models::{HcaUser, db::User};

#[derive(Clone)]
pub(crate) struct UsersService {
    db: PgPool,
}

impl UsersService {
    pub(crate) fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub(crate) async fn upsert_with(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        user: &HcaUser,
        access_token: &str,
        refresh_token: &str,
        token_expires_at: DateTime<Utc>,
    ) -> Result<String> {
        // Look up existing user by email to link social accounts
        let existing: Option<(String,)> = query_as("SELECT id FROM users WHERE email = $1 LIMIT 1")
            .bind(&user.primary_email)
            .fetch_optional(&mut **tx)
            .await?;

        let user_id = if let Some((existing_id,)) = existing {
            query(
                r#"
                UPDATE users SET
                    first_name = $1,
                    last_name = $2,
                    verification_status = $3,
                    ysws_eligible = $4,
                    updated_at = NOW(),
                    hca_access_token = $5,
                    hca_refresh_token = $6,
                    hca_token_expires_at = $7
                WHERE id = $8
                "#,
            )
            .bind(&user.first_name)
            .bind(&user.last_name)
            .bind(&user.verification_status)
            .bind(user.ysws_eligible)
            .bind(access_token)
            .bind(refresh_token)
            .bind(token_expires_at)
            .bind(&existing_id)
            .execute(&mut **tx)
            .await?;
            existing_id
        } else {
            let new_id = format!("hf!{}", Uuid::new_v4());
            query(
                r#"
                INSERT INTO users (id, email, first_name, last_name, verification_status, ysws_eligible, hca_access_token, hca_refresh_token, hca_token_expires_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(&new_id)
            .bind(&user.primary_email)
            .bind(&user.first_name)
            .bind(&user.last_name)
            .bind(&user.verification_status)
            .bind(user.ysws_eligible)
            .bind(access_token)
            .bind(refresh_token)
            .bind(token_expires_at)
            .execute(&mut **tx)
            .await?;
            new_id
        };

        Ok(user_id)
    }

    pub(crate) async fn get_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = query_as::<_, User>("SELECT * from users WHERE id = $1 LIMIT 1")
            .bind(id)
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }
}
