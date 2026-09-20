use askama::Template;
use askama_web::WebTemplate;

use crate::{
    frontend::api::friendly_error,
    frontend::models::{
        AdminStats, AdminUser, ApiKey, AuthenticatedUser, ConfigEntry, CreatedApiKey, DnsRecord,
        DnsZone, Notification, QueryLogEntry, QueryLogsSummary, TimeseriesPoint, TopQuery,
        TrafficSummary, UserSession, ZoneTraffic,
    },
};

// --- Public pages ---

#[derive(Template, WebTemplate)]
#[template(path = "home.html")]
pub struct HomeTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
    pub email: String,
    pub hackclub_login_url: String,
}

impl LoginTemplate {
    pub fn new(email: String, hackclub_login_url: String) -> Self {
        Self {
            error: None,
            email,
            hackclub_login_url,
        }
    }

    /// Render with a user-facing error derived from a backend error code.
    pub fn with_error(mut self, error_code: impl AsRef<str>) -> Self {
        self.error = Some(friendly_error(error_code.as_ref()));
        self
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "forgot_password.html")]
pub struct ForgotPasswordTemplate {
    pub error: Option<String>,
    pub message: Option<String>,
    pub email: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "reset_password.html")]
pub struct ResetPasswordTemplate {
    pub error: Option<String>,
    pub token: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub status: u16,
    pub message: String,
    pub details: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "public_page.html")]
pub struct PublicPageTemplate {
    pub title: String,
    pub heading: String,
    pub message: String,
    pub status: u16,
}

// --- Dashboard ---

/// Context shared by every dashboard template
#[derive(Debug, Clone)]
pub struct DashContext {
    pub page_title: String,
    pub active: String,
    pub display_name: String,
    pub email: String,
    pub initials: String,
    pub is_admin: bool,
}

impl DashContext {
    pub fn from_user(user: &AuthenticatedUser, active: &str, page_title: &str) -> Self {
        let full_name = format!("{} {}", user.first_name, user.last_name);
        let display_name = if full_name.trim().is_empty() {
            user.email.clone()
        } else {
            full_name.trim().to_string()
        };
        let initials = initials_from(&user.first_name, &user.last_name, &user.email);
        Self {
            page_title: page_title.to_string(),
            active: active.to_string(),
            display_name,
            email: user.email.clone(),
            initials,
            is_admin: user.is_admin,
        }
    }
}

fn initials_from(first_name: &str, last_name: &str, email: &str) -> String {
    let mut out = String::new();
    for part in [first_name, last_name]
        .into_iter()
        .filter(|p| !p.is_empty())
    {
        if let Some(c) = part.chars().next() {
            out.push(c.to_ascii_uppercase());
        }
    }
    if out.is_empty()
        && let Some(c) = email.chars().next()
    {
        out.push(c.to_ascii_uppercase());
    }
    out
}

#[derive(Template, WebTemplate)]
#[template(path = "dash/base.html")]
pub struct DashboardTemplate {
    pub page_title: String,
    pub active: String,
    pub display_name: String,
    pub email: String,
    pub initials: String,
    pub is_admin: bool,
    pub domain: Option<String>,
    pub title: String,
    pub description: String,
    pub zones: Vec<DnsZone>,
    pub verified_count: usize,
    pub pending_count: usize,
    pub zone_name: String,
    pub records: Vec<DnsRecord>,
    pub recent_logs: Vec<QueryLogEntry>,
    pub ns_verified: bool,
    pub a_count: usize,
    pub cname_count: usize,
    pub other_count: usize,
    pub summary: Option<TrafficSummary>,
    pub timeseries: Vec<TimeseriesPoint>,
    pub by_zone: Vec<ZoneTraffic>,
    pub top_queries: Vec<TopQuery>,
    pub logs: Vec<QueryLogEntry>,
    pub log_summary: QueryLogsSummary,
    pub log_zone: String,
    pub notifications: Vec<Notification>,
    pub keys: Vec<ApiKey>,
    pub created_key: Option<CreatedApiKey>,
    pub user: AuthenticatedUser,
    pub sessions: Vec<UserSession>,
    pub slack_id: Option<String>,
    pub config: Vec<ConfigEntry>,
    pub users: Vec<AdminUser>,
    pub stats: Option<AdminStats>,
    pub traffic_summary: Option<TrafficSummary>,
    pub traffic_timeseries: Vec<TimeseriesPoint>,
    pub message: Option<String>,
    pub error: Option<String>,
}
