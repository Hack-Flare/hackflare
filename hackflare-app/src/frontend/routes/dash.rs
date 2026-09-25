//! Dashboard routes. Every page here is an auth-guarded, server-rendered page

use axum::{
    extract::{Form, Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    api::routes::{
        dns::{self as dns_api, DnsActionError},
        settings as settings_api,
    },
    api::services::notifications as notifications_service,
    auth::middleware,
    frontend::models::{
        ApiKey, AuthenticatedUser, DnsRecord, DnsZone, Notification, QueryLogsSummary,
    },
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

    let mut page = dashboard_page(
        &user,
        "dashboard",
        "Dashboard",
        &state.config.dns_nameservers,
    );
    page.zones = zones;
    page.verified_count = verified_count;
    page.into_response()
}

/// Result of a dashboard form submission, carried across the post/redirect/get
/// round trip as a fixed code so arbitrary text can never be injected.
#[derive(Debug, Default, Deserialize)]
pub struct Flash {
    notice: Option<String>,
    error: Option<String>,
}

impl Flash {
    fn apply(self, page: &mut DashboardTemplate) {
        page.message = self.notice.as_deref().and_then(notice_text).map(str::to_string);
        page.error = self.error.as_deref().and_then(error_text).map(str::to_string);
    }
}

fn notice_text(code: &str) -> Option<&'static str> {
    Some(match code {
        "domain_added" => "Domain added. Point its nameservers at HackFlare to verify it.",
        "domain_deleted" => "Domain deleted.",
        "propagation_verified" => "Nameserver propagation confirmed. DNS records are now available.",
        "record_added" => "Record added.",
        "record_deleted" => "Record deleted.",
        "password_updated" => "Password updated.",
        "notifications_read" => "All notifications marked as read.",
        _ => return None,
    })
}

fn error_text(code: &str) -> Option<&'static str> {
    Some(match code {
        "name_required" => "Please enter a name.",
        "zone_exists" => "That domain has already been added.",
        "invalid_zone" => "That isn't a valid domain name.",
        "zone_not_found" => "Domain not found.",
        "propagation_pending" => {
            "Nameservers are not fully propagated yet. Check your registrar settings and try again."
        }
        "zone_not_verified" => {
            "This domain isn't verified yet, so its records can't be changed."
        }
        "record_not_found" => "Record not found.",
        "invalid_ttl" => "TTL must be a whole number of seconds.",
        "password_too_short" => "Password must be at least 8 characters.",
        "password_mismatch" => "The new passwords don't match.",
        "current_password_incorrect" => "Your current password is incorrect.",
        "invalid_request" => "That request wasn't understood.",
        "internal" => "Something went wrong. Please try again.",
        _ => return None,
    })
}

fn dns_error_code(error: DnsActionError) -> &'static str {
    match error {
        DnsActionError::NameRequired => "name_required",
        DnsActionError::ZoneExists => "zone_exists",
        DnsActionError::InvalidZone => "invalid_zone",
        DnsActionError::ZoneNotFound => "zone_not_found",
        DnsActionError::ZoneNotVerified => "zone_not_verified",
        DnsActionError::RecordNotFound => "record_not_found",
        DnsActionError::Internal => "internal",
    }
}

fn redirect_with(path: &str, result: Result<&str, &str>) -> Response {
    let query = match result {
        Ok(code) => format!("notice={code}"),
        Err(code) => format!("error={code}"),
    };
    Redirect::to(&format!("{path}?{query}")).into_response()
}

/// Characters escaped when a zone name is placed in a redirect path segment.
const PATH_SEGMENT: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn domain_dns_path(zone_name: &str) -> String {
    format!(
        "/dash/domains/{}/dns",
        percent_encoding::utf8_percent_encode(zone_name, PATH_SEGMENT)
    )
}

/// Generic placeholder page for each sidebar section.
pub async fn section(
    State(state): State<AppState>,
    Path(section): Path<String>,
    Query(flash): Query<Flash>,
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

    // The admin panel is only for admins; hide its existence from everyone else.
    if *key == "admin" && !user.is_admin {
        return not_found().await;
    }

    let mut page = dashboard_page(&user, key, title, &state.config.dns_nameservers);
    page.description = description.to_string();
    flash.apply(&mut page);

    match *key {
        "domains" => {
            page.zones = fetch_zones(&state, &user.id).await;
            page.verified_count = page.zones.iter().filter(|zone| zone.ns_verified).count();
            page.pending_count = page.zones.len() - page.verified_count;
        }
        "notifications" => {
            page.notifications =
                notifications_service::list_notifications(&state.db, &user.id, 50)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|n| Notification {
                        id: n.id.to_string(),
                        user_id: n.user_id,
                        title: n.title,
                        message: n.message,
                        notification_type: n.notif_type,
                        read: n.read,
                        link: n.link,
                        created_at: n.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
                    })
                    .collect();
        }
        "settings" => {
            page.keys = state
                .api_keys
                .list(&user.id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|k| ApiKey {
                    id: k.id.to_string(),
                    name: k.name,
                    prefix: k.prefix,
                    created_at: k.created_at.to_rfc3339(),
                    last_used_at: k.last_used_at.map(|t| t.to_rfc3339()),
                    revoked: k.revoked_at.is_some(),
                })
                .collect();
        }
        _ => {}
    }

    page.into_response()
}

/// Zone-scoped sub-pages (`/dash/domains/:domain/{dns,ssl,redirects}`).
pub async fn domain_sub(
    State(state): State<AppState>,
    Path((domain, sub)): Path<(String, String)>,
    Query(flash): Query<Flash>,
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
    let mut page = dashboard_page(&user, &sub, title, &state.config.dns_nameservers);
    page.zone_name = domain.clone();
    page.description = description;
    flash.apply(&mut page);

    if sub == "dns" {
        let (verified, records) = match dns_api::zone_records(&state, &user.id, &domain).await {
            Ok(result) => result,
            Err(DnsActionError::ZoneNotFound) => return not_found().await,
            Err(_) => (false, Vec::new()),
        };
        page.ns_verified = verified;
        page.a_count = records.iter().filter(|r| r.r#type == "A").count();
        page.cname_count = records.iter().filter(|r| r.r#type == "CNAME").count();
        page.other_count = records.len() - page.a_count - page.cname_count;
        page.records = records
            .into_iter()
            .map(|r| DnsRecord {
                id: r.id,
                name: r.name,
                record_type: r.r#type,
                value: r.value,
                ttl: i64::from(r.ttl),
                status: r.status,
            })
            .collect();
    }

    page.domain = Some(domain);
    page.into_response()
}

fn dashboard_page(
    user: &AuthenticatedUser,
    active: &str,
    title: &str,
    nameservers: &[String],
) -> DashboardTemplate {
    let ctx = DashContext::from_user(user, active, title);
    DashboardTemplate {
        page_title: ctx.page_title,
        active: ctx.active,
        display_name: ctx.display_name,
        email: ctx.email,
        initials: ctx.initials,
        is_admin: ctx.is_admin,
        domain: None,
        nameservers: nameservers.to_vec(),
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
        users: Vec::new(),
        stats: None,
        traffic_summary: None,
        traffic_timeseries: Vec::new(),
        message: None,
        error: None,
    }
}

// --- Form pages ---
//
// These paths get explicit routes so they can also accept POSTs; their GETs
// render exactly like the generic section/sub-page handlers.

pub async fn domains_get(
    state: State<AppState>,
    flash: Query<Flash>,
    headers: HeaderMap,
) -> Response {
    section(state, Path("domains".to_string()), flash, headers).await
}

pub async fn settings_get(
    state: State<AppState>,
    flash: Query<Flash>,
    headers: HeaderMap,
) -> Response {
    section(state, Path("settings".to_string()), flash, headers).await
}

pub async fn notifications_get(
    state: State<AppState>,
    flash: Query<Flash>,
    headers: HeaderMap,
) -> Response {
    section(state, Path("notifications".to_string()), flash, headers).await
}

pub async fn dns_get(
    state: State<AppState>,
    Path(domain): Path<String>,
    flash: Query<Flash>,
    headers: HeaderMap,
) -> Response {
    domain_sub(state, Path((domain, "dns".to_string())), flash, headers).await
}

// --- Form submissions ---

#[derive(Debug, Deserialize)]
pub struct DomainForm {
    action: String,
    name: String,
}

/// `POST /dash/domains`: add or delete a domain.
pub async fn domains_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<DomainForm>,
) -> Response {
    let user = match require_user(&state, &headers).await {
        Ok(user) => user,
        Err(redirect) => return redirect.into_response(),
    };

    let result = match form.action.as_str() {
        "add" => dns_api::add_zone(&state, &user.id, &form.name)
            .await
            .map(|_| "domain_added")
            .map_err(dns_error_code),
        "delete" => dns_api::remove_zone(&state, &user.id, &form.name)
            .await
            .map(|()| "domain_deleted")
            .map_err(dns_error_code),
        "check" => {
            if dns_api::ensure_zone_ownership(&state.db, &form.name, &user.id)
                .await
                .is_err()
            {
                return redirect_with("/dash/domains", Err("zone_not_found"));
            }
            let result = dns_api::check_zone_propagation(&state, &form.name).await.0;
            if result
                .get("verified")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            {
                Ok("propagation_verified")
            } else {
                Err("propagation_pending")
            }
        }
        _ => return redirect_with("/dash/domains", Err("invalid_request")),
    };
    redirect_with("/dash/domains", result)
}

#[derive(Debug, Deserialize)]
pub struct DnsForm {
    action: String,
    id: Option<String>,
    name: Option<String>,
    record_type: Option<String>,
    value: Option<String>,
    ttl: Option<String>,
}

/// `POST /dash/domains/:domain/dns`: add or delete a record.
pub async fn dns_post(
    State(state): State<AppState>,
    Path(domain): Path<String>,
    headers: HeaderMap,
    Form(form): Form<DnsForm>,
) -> Response {
    let user = match require_user(&state, &headers).await {
        Ok(user) => user,
        Err(redirect) => return redirect.into_response(),
    };
    let back = domain_dns_path(&domain);

    let result = match form.action.as_str() {
        "create" => {
            let ttl = match form.ttl.as_deref().map(str::trim) {
                None | Some("") => 3600,
                Some(raw) => match raw.parse::<u32>() {
                    Ok(ttl) => ttl,
                    Err(_) => return redirect_with(&back, Err("invalid_ttl")),
                },
            };
            let name = form.name.unwrap_or_default();
            let value = form.value.unwrap_or_default();
            if name.trim().is_empty() || value.trim().is_empty() {
                return redirect_with(&back, Err("name_required"));
            }
            dns_api::add_record(
                &state,
                &user.id,
                &domain,
                name.trim(),
                form.record_type.as_deref().unwrap_or("A"),
                ttl,
                value.trim(),
            )
            .await
            .map(|()| "record_added")
        }
        "delete" => {
            let Some(record_id) = form.id.as_deref().and_then(|id| id.parse().ok()) else {
                return redirect_with(&back, Err("record_not_found"));
            };
            dns_api::remove_record(&state, &user.id, &domain, record_id)
                .await
                .map(|()| "record_deleted")
        }
        _ => return redirect_with(&back, Err("invalid_request")),
    };
    redirect_with(&back, result.map_err(dns_error_code))
}

#[derive(Debug, Deserialize)]
pub struct SettingsForm {
    action: String,
    current_password: Option<String>,
    new_password: String,
    confirm_password: String,
}

/// `POST /dash/settings`: change the account password.
pub async fn settings_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<SettingsForm>,
) -> Response {
    // The password check needs the stored hash, so load the full user record.
    let Some(user) = middleware::user_from_headers(&state, &headers).await else {
        return Redirect::to("/login").into_response();
    };

    if form.action != "password" {
        return redirect_with("/dash/settings", Err("invalid_request"));
    }
    if form.new_password != form.confirm_password {
        return redirect_with("/dash/settings", Err("password_mismatch"));
    }

    let result = settings_api::change_password(
        &state,
        &user,
        form.current_password.as_deref(),
        &form.new_password,
    )
    .await
    .map(|()| "password_updated")
    .map_err(|code| match code {
        "password_too_short" | "current_password_incorrect" => code,
        _ => "internal",
    });
    redirect_with("/dash/settings", result)
}

#[derive(Debug, Deserialize)]
pub struct NotificationsForm {
    action: String,
}

/// `POST /dash/notifications`: mark every notification as read.
pub async fn notifications_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<NotificationsForm>,
) -> Response {
    let user = match require_user(&state, &headers).await {
        Ok(user) => user,
        Err(redirect) => return redirect.into_response(),
    };

    if form.action != "all" {
        return redirect_with("/dash/notifications", Err("invalid_request"));
    }
    let result = notifications_service::mark_all_read(&state.db, &user.id)
        .await
        .map(|()| "notifications_read")
        .map_err(|_| "internal");
    redirect_with("/dash/notifications", result)
}
