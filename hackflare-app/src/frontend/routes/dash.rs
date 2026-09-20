//! Dashboard routes. Every page here is an auth-guarded, server-rendered page

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};

use crate::{
    frontend::models::{AuthenticatedUser, DnsZone, QueryLogsSummary},
    frontend::pages::{DashContext, DashboardTemplate},
    state::AppState,
};

use super::handlers::{not_found, require_user};

/// (route key, page title, section description) for sidebar sections.
const SECTION_META: &[(&str, &str, &str)] = &[
    ("domains", "Domains", "Manage and register your domains."),
    (
        "firewall",
        "Firewall",
        "Protect your edge with firewall rules.",
    ),
    ("workers", "Workers", "Deploy scripts at the edge."),
    ("tunnel", "Tunnel", "Expose local services securely."),
    ("traffic", "Traffic", "DNS traffic analytics and insights."),
    (
        "performance",
        "Performance",
        "Performance monitoring and insights.",
    ),
    ("logs", "Logs", "Query and request logs."),
    (
        "notifications",
        "Notifications",
        "Alerts and account notifications.",
    ),
    ("settings", "Settings", "Account and workspace settings."),
    ("profile", "Profile", "Your profile and preferences."),
    ("admin", "Admin Panel", "Platform administration."),
    ("help", "Help", "Support and documentation."),
];

const DOMAIN_SUB_META: &[(&str, &str)] = &[
    ("dns", "DNS Records"),
    ("ssl", "SSL/TLS"),
    ("redirects", "Redirects"),
];

async fn fetch_zones(state: &AppState, user_id: &str) -> Vec<DnsZone> {
    sqlx::query_as::<_, (String, bool)>(
        "SELECT name, ns_verified FROM dns_zones WHERE user_id = $1 ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(name, ns_verified)| DnsZone { name, ns_verified })
    .collect()
}

/// Dashboard home, the representative SSR page.
pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match require_user(&state, &headers).await {
        Ok(user) => user,
        Err(redirect) => return redirect.into_response(),
    };

    let zones = fetch_zones(&state, &user.id).await;
    let verified_count = zones.iter().filter(|zone| zone.ns_verified).count();

    let mut page = dashboard_page(&user, "dashboard", "Dashboard");
    page.zones = zones;
    page.verified_count = verified_count;
    page.into_response()
}

/// Generic placeholder page for each sidebar section.
pub async fn section(
    State(state): State<AppState>,
    Path(section): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some((key, title, description)) = SECTION_META.iter().find(|(key, _, _)| *key == section)
    else {
        return not_found().await;
    };

    let user = match require_user(&state, &headers).await {
        Ok(user) => user,
        Err(redirect) => return redirect.into_response(),
    };

    let mut page = dashboard_page(&user, key, title);
    page.description = description.to_string();
    page.into_response()
}

/// Zone-scoped sub-pages (`/dash/domains/:domain/{dns,ssl,redirects}`).
pub async fn domain_sub(
    State(state): State<AppState>,
    Path((domain, sub)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let Some((_, title)) = DOMAIN_SUB_META.iter().find(|(key, _)| *key == sub) else {
        return not_found().await;
    };

    let user = match require_user(&state, &headers).await {
        Ok(user) => user,
        Err(redirect) => return redirect.into_response(),
    };

    let description = format!("Manage {title} for {domain}.");
    let mut page = dashboard_page(&user, &sub, title);
    page.domain = Some(domain);
    page.description = description;
    page.into_response()
}

fn dashboard_page(user: &AuthenticatedUser, active: &str, title: &str) -> DashboardTemplate {
    let ctx = DashContext::from_user(user, active, title);
    DashboardTemplate {
        page_title: ctx.page_title,
        active: ctx.active,
        display_name: ctx.display_name,
        email: ctx.email,
        initials: ctx.initials,
        is_admin: ctx.is_admin,
        domain: None,
        title: title.to_string(),
        description: String::new(),
        zones: Vec::new(),
        verified_count: 0,
        pending_count: 0,
        zone_name: String::new(),
        records: Vec::new(),
        recent_logs: Vec::new(),
        ns_verified: false,
        a_count: 0,
        cname_count: 0,
        other_count: 0,
        summary: None,
        timeseries: Vec::new(),
        by_zone: Vec::new(),
        top_queries: Vec::new(),
        logs: Vec::new(),
        log_summary: QueryLogsSummary {
            errors_today: 0,
            warnings_today: 0,
            info_today: 0,
        },
        log_zone: String::new(),
        notifications: Vec::new(),
        keys: Vec::new(),
        created_key: None,
        user: user.clone(),
        sessions: Vec::new(),
        config: Vec::new(),
        users: Vec::new(),
        stats: None,
        traffic_summary: None,
        traffic_timeseries: Vec::new(),
        message: None,
        error: None,
    }
}
