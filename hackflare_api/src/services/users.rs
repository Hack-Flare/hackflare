use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::routes::auth::HcaUser;

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
        token_expires_in: DateTime<Utc>,
    ) -> Result<()> {
        todo!("upsert user")
    }
}
