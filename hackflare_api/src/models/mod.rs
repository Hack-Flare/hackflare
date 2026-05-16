use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{TimestampSeconds, serde_as};
use uuid::Uuid;

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HcaUser {
    /// Unique ID of a user.
    pub(crate) id: String,

    /// Is this user eligible for YSWS programs? In other words, are they eligible for Hackflare?
    pub(crate) ysws_eligible: bool,
    pub(crate) verification_status: String,

    /// User's legal first name.
    pub(crate) first_name: String,

    /// User's legal last name.
    pub(crate) last_name: String,

    /// The primary - and only - email of the user.
    pub(crate) primary_email: String,

    /// The Slack ID of the user.
    pub(crate) slack_id: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JwtClaims {
    pub(crate) sub: String,
    pub(crate) jit: Uuid,
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub(crate) exp: DateTime<Utc>,
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub(crate) iat: DateTime<Utc>,
}

#[derive(Clone, Serialize)]
pub(crate) struct CurrentUser {
    pub(crate) user: db::User,
    pub(crate) session: db::UserSession,
}
