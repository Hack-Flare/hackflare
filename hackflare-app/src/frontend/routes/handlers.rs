//! Handlers for the public (non-dashboard) server-rendered pages.
//!
//! POST handlers call the backend directly

use askama::Template;
use axum::{
    extract::{ConnectInfo, Form, Json, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::net::SocketAddr;
use axum_client_ip::ClientIp;
use axum_extra::extract::CookieJar;

use crate::{
    auth::{middleware, routes as auth_routes},
    frontend::pages::{
        ErrorTemplate, ForgotPasswordTemplate, HomeTemplate, LoginTemplate, RegisterTemplate,
        ResetPasswordTemplate,
    },
    state::AppState,
};

// --- Helpers ---

fn attach_cookies(response: &mut Response, source: &Response) {
    for value in source.headers().get_all(header::SET_COOKIE).iter() {
        response.headers_mut().append(header::SET_COOKIE, value.clone());
    }
}

/// Request origin derived from the standard proxy headers.
fn origin_for(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|proto| *proto == "https")
        .unwrap_or("http");
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("localhost:3000");
    format!("{scheme}://{host}")
}

fn encode_component(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Hack Club auth entry point
fn hackclub_target_url(headers: &HeaderMap, return_to: &str) -> String {
    let target = format!(
        "{}/auth/hackclub?returnTo={}",
        origin_for(headers),
        encode_component(return_to)
    );
    format!(
        "/api/v1/auth/login?target={}",
        encode_component(&target)
    )
}

/// Sanitize a user-supplied `returnTo` so it can never be an open redirect.
fn safe_return_to(return_to: Option<String>) -> String {
    match return_to {
        Some(value)
            if value.starts_with('/')
                && !value.starts_with("//")
                && !value.contains('\\')
                && !value.contains(':') =>
        {
            value
        }
        _ => "/dash".to_string(),
    }
}

/// Bail out to `/login` when the incoming cookies don't describe a session.
pub async fn require_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::frontend::models::AuthenticatedUser, Redirect> {
    let Some(user) = middleware::user_from_headers(state, headers).await else {
        return Err(Redirect::to("/login"));
    };
    let email = user.email.clone();
    Ok(crate::frontend::models::AuthenticatedUser {
        id: user.id,
        first_name: user.first_name,
        last_name: user.last_name,
        email,
        eligible: user.ysws_eligible,
        has_password: user.password_hash.is_some(),
        is_admin: state.config.admin_emails.iter().any(|email| email == &user.email),
        created_at: user.created_at.to_rfc3339(),
    })
}

pub fn render_error(status: u16, message: &str, details: &str) -> Response {
    let template = ErrorTemplate {
        status,
        message: message.to_string(),
        details: details.to_string(),
    };
    match Template::render(&template) {
        Ok(html) => (
            StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Html(html),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "failed to render error template");
            (StatusCode::INTERNAL_SERVER_ERROR, "template error").into_response()
        }
    }
}

// --- Home page ---

/// Fallback for the API examples when `FRONTEND_URL` is unset.
const PUBLIC_ORIGIN_FALLBACK: &str = "https://hackflare.net";

pub async fn home(State(state): State<AppState>) -> HomeTemplate {
    let api_base_url = state
        .config
        .frontend_url
        .as_ref()
        .map(|url| url.as_str().trim_end_matches('/').to_string())
        .unwrap_or_else(|| PUBLIC_ORIGIN_FALLBACK.to_string());
    let api_host = api_base_url
        .split_once("://")
        .map_or(api_base_url.as_str(), |(_, host)| host)
        .to_string();

    HomeTemplate {
        api_base_url,
        api_host,
    }
}

pub async fn auth_redirect() -> Redirect {
    Redirect::to("/login")
}

// --- Login ---

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn login_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if middleware::user_from_headers(&state, &headers).await.is_some() {
        return Redirect::to("/dash").into_response();
    }
    LoginTemplate::new(String::new(), hackclub_target_url(&headers, "/dash")).into_response()
}

pub async fn login_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Form(form): Form<LoginForm>,
) -> Response {
    let template = LoginTemplate::new(form.email.clone(), hackclub_target_url(&headers, "/dash"));
    match auth_routes::email_login_handler(
        State(state),
        ClientIp(address.ip()),
        Json(auth_routes::LoginRequest { email: form.email, password: form.password }),
    )
    .await
    {
        Ok(auth_response) if auth_response.status().is_success() => {
            let mut response = Redirect::to("/dash").into_response();
            attach_cookies(&mut response, &auth_response);
            response
        }
        Err((_, code)) => template.with_error(code).into_response(),
        Ok(_) => template
            .with_error("Something went wrong. Please try again.")
            .into_response(),
    }
}

// --- Register ---

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
}

pub async fn register_get() -> RegisterTemplate {
    RegisterTemplate {
        error: None,
        first_name: String::new(),
        last_name: String::new(),
        email: String::new(),
    }
}

pub async fn register_post(
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Form(form): Form<RegisterForm>,
) -> Response {
    match auth_routes::register_handler(
        State(state),
        ClientIp(address.ip()),
        Json(auth_routes::RegisterRequest {
            email: form.email.clone(),
            password: form.password,
            first_name: form.first_name.clone(),
            last_name: form.last_name.clone(),
        }),
    )
    .await
    {
        Ok(auth_response) if auth_response.status().is_success() => {
            let mut response = Redirect::to("/dash").into_response();
            attach_cookies(&mut response, &auth_response);
            response
        }
        Err((_, code)) => {
            RegisterTemplate {
                error: Some(code.to_string()),
                first_name: form.first_name,
                last_name: form.last_name,
                email: form.email,
            }
            .into_response()
        }
        Ok(_) => RegisterTemplate {
            error: Some("Something went wrong. Please try again.".to_string()),
            first_name: form.first_name,
            last_name: form.last_name,
            email: form.email,
        }
        .into_response(),
    }
}

// --- Forgot password ---

#[derive(Debug, Deserialize)]
pub struct ForgotForm {
    email: String,
}

pub async fn forgot_get() -> ForgotPasswordTemplate {
    ForgotPasswordTemplate {
        error: None,
        message: None,
        email: String::new(),
    }
}

pub async fn forgot_post(State(state): State<AppState>, Form(form): Form<ForgotForm>) -> Response {
    match auth_routes::forgot_password_handler(
        State(state),
        Json(auth_routes::ForgotPasswordRequest { email: form.email.clone() }),
    )
    .await
    {
        Ok(_) => ForgotPasswordTemplate {
            error: None,
            message: Some(
                "If an account exists for that email, a reset link has been sent.".to_string(),
            ),
            email: String::new(),
        }
        .into_response(),
        Err((_, code)) => {
            ForgotPasswordTemplate {
                error: Some(code.to_string()),
                message: None,
                email: form.email,
            }
            .into_response()
        }
    }
}

// --- Reset password ---

#[derive(Debug, Deserialize)]
pub struct ResetForm {
    password: String,
    password_confirm: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetQuery {
    token: String,
}

pub async fn reset_get(Query(query): Query<ResetQuery>) -> ResetPasswordTemplate {
    ResetPasswordTemplate {
        error: None,
        token: query.token,
    }
}

pub async fn reset_post(
    State(state): State<AppState>,
    Query(query): Query<ResetQuery>,
    Form(form): Form<ResetForm>,
) -> Response {
    let template = ResetPasswordTemplate {
        error: None,
        token: query.token.clone(),
    };

    if form.password != form.password_confirm {
        return ResetPasswordTemplate {
            error: Some("Passwords do not match.".to_string()),
            ..template
        }
        .into_response();
    }

    match auth_routes::reset_password_handler(
        State(state),
        Json(auth_routes::ResetPasswordRequest {
            token: query.token,
            password: form.password,
        }),
    )
    .await
    {
        Ok(_) => Redirect::to("/login").into_response(),
        Err((_, code)) => {
            ResetPasswordTemplate {
                error: Some(code.to_string()),
                ..template
            }
            .into_response()
        }
    }
}

// --- Hack Club auth ---

#[derive(Debug, Deserialize)]
pub struct HackClubParams {
    return_to: Option<String>,
}

pub async fn hackclub(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HackClubParams>,
) -> Response {
    let return_to = safe_return_to(params.return_to);
    if middleware::user_from_headers(&state, &headers).await.is_some() {
        return Redirect::to(&return_to).into_response();
    }
    Redirect::to("/login").into_response()
}

// --- Logout ---

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let jar = CookieJar::from_headers(&headers);
    let auth_response = auth_routes::logout_handler(
        State(state.clone()),
        State(state.user_sessions.clone()),
        jar,
    )
    .await;
    let mut response = Redirect::to("/").into_response();
    attach_cookies(&mut response, &auth_response);
    response
}

// --- Health / errors ---

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

pub async fn not_found() -> Response {
    render_error(
        404,
        "Page not found",
        "The page you are looking for doesn't exist or has moved.",
    )
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use super::{hackclub_target_url, safe_return_to};

    #[test]
    fn hackclub_target_starts_oauth_flow() {
        let target = hackclub_target_url(&HeaderMap::new(), "/dash");
        assert!(target.starts_with("/api/v1/auth/login?target="));
    }

    #[test]
    fn safe_return_to_keeps_local_paths() {
        assert_eq!(safe_return_to(Some("/dash/domains".into())), "/dash/domains");
    }

    #[test]
    fn safe_return_to_rejects_external_targets() {
        for value in [
            "//attacker.example",
            "/\\attacker.example",
            "https://attacker.example",
            "dash",
        ] {
            assert_eq!(safe_return_to(Some(value.into())), "/dash", "{value}");
        }
        assert_eq!(safe_return_to(None), "/dash");
    }
}
