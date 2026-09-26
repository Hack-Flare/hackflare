#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub eligible: bool,
    pub has_password: bool,
    pub is_admin: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsZone {
    pub name: String,
    pub ns_verified: bool,
    /// Records in this zone, shown on the dashboard overview.
    pub record_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub value: String,
    pub ttl: i64,
    pub status: String,
}

impl DnsRecord {
    /// TTL as the records table shows it: `auto`, `5m`, `1h`, `2d`.
    pub fn ttl_human(&self) -> String {
        match self.ttl {
            ..=0 => "auto".to_string(),
            ttl if ttl % 86_400 == 0 => format!("{}d", ttl / 86_400),
            ttl if ttl % 3_600 == 0 => format!("{}h", ttl / 3_600),
            ttl if ttl % 60 == 0 => format!("{}m", ttl / 60),
            ttl => format!("{ttl}s"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub ip_address: String,
    pub expires_at: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub dns_zones: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub path: String,
    pub zone: String,
    pub status: i64,
    pub ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogsResponse {
    pub logs: Vec<QueryLogEntry>,
    pub summary: QueryLogsSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogsSummary {
    pub errors_today: i64,
    pub warnings_today: i64,
    pub info_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_zones: i64,
    pub total_sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub message: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub read: bool,
    pub link: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCount {
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSummary {
    pub total_requests: i64,
    pub avg_processing_ms: i64,
    pub success_rate: i64,
    pub error_rate: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesPoint {
    pub date: String,
    pub requests: i64,
    pub errors: i64,
    pub nxdomain: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneTraffic {
    pub zone: String,
    pub requests: i64,
    pub errors: i64,
    pub avg_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopQuery {
    pub query: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub revoked: bool,
    /// "Just now" for today, otherwise a short date like "Aug 30".
    pub created_label: String,
    /// "Never", "2 hours ago", "3 months ago".
    pub last_used_label: String,
    /// Unused for 60+ days, so the token list can call it out.
    pub last_used_stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedApiKey {
    pub key: ApiKey,
    pub raw_key: String,
}
