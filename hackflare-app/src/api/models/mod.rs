use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

}

mod timestamp_seconds {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(value.timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = i64::deserialize(deserializer)?;
        DateTime::from_timestamp(seconds, 0)
            .ok_or_else(|| serde::de::Error::custom("timestamp is out of range"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub jit: Uuid,
    #[serde(with = "timestamp_seconds")]
    pub exp: DateTime<Utc>,
    #[serde(with = "timestamp_seconds")]
    pub iat: DateTime<Utc>,
    pub typ: Option<String>,
}

#[derive(Clone, Serialize)]
pub(crate) struct CurrentUser {
    pub(crate) user: db::User,
    pub(crate) session: db::UserSession,
}
