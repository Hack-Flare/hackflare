//! Dashboard routes. Every page here is an auth-guarded, server-rendered page

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};

use crate::{
    api,
    models::DnsZone,
    pages::{DashContext, DashIndexTemplate, DashSectionTemplate},
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

async fn fetch_zones(state: &AppState, cookie: Option<&HeaderValue>) -> Vec<DnsZone> {
    let url = api::api_url(&state.config.api_proxy_target, "/dns/zones");
    let mut request = state.http_client.get(url);
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }

    let Ok(response) = request.send().await else {
        return Vec::new();
    };
    if !response.status().is_success() {
        return Vec::new();
    }
    response.json::<Vec<DnsZone>>().await.unwrap_or_default()
}

/// Dashboard home, the representative SSR page.
pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match require_user(&state, &headers).await {
        Ok(user) => user,
        Err(redirect) => return redirect.into_response(),
    };

    let cookie = headers.get(header::COOKIE).cloned();
    let zones = fetch_zones(&state, cookie.as_ref()).await;
    let verified_count = zones.iter().filter(|zone| zone.ns_verified).count();

    let ctx = DashContext::from_user(&user, "dashboard", "Dashboard");
    DashIndexTemplate {
        page_title: ctx.page_title,
        active: ctx.active,
        display_name: ctx.display_name,
        email: ctx.email,
        initials: ctx.initials,
        is_admin: ctx.is_admin,
        domain: None,
        zones,
        verified_count,
    }
    .into_response()
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

    let ctx = DashContext::from_user(&user, key, title);
    DashSectionTemplate {
        page_title: ctx.page_title,
        active: ctx.active,
        display_name: ctx.display_name,
        email: ctx.email,
        initials: ctx.initials,
        is_admin: ctx.is_admin,
        domain: None,
        title: title.to_string(),
        description: description.to_string(),
    }
    .into_response()
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

    let ctx = DashContext::from_user(&user, &sub, title);
    let description = format!("Manage {title} for {domain}.");
    DashSectionTemplate {
        page_title: ctx.page_title,
        active: ctx.active,
        display_name: ctx.display_name,
        email: ctx.email,
        initials: ctx.initials,
        is_admin: ctx.is_admin,
        domain: Some(domain),
        title: title.to_string(),
        description,
    }
    .into_response()
}
