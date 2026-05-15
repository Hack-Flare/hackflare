use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub slack_id: String,
    pub first_name: String,
    pub last_name: String,
    pub verification_status: String,
    pub ysws_eligible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // HCA Access Data
    pub hca_access_token: String,
    pub hca_refresh_token: String,
    pub hca_token_expires_at: DateTime<Utc>,
}
