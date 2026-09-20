//! Handlers for the public (non-dashboard) server-rendered pages.
//!
//! POST handlers call the backend directly

use askama::Template;
use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    frontend::api,
    frontend::pages::{
        ErrorTemplate, ForgotPasswordTemplate, HomeTemplate, LoginTemplate, RegisterTemplate,
        ResetPasswordTemplate,
    },
    state::AppState,
};

// --- Helpers ---

/// `Set-Cookie` values from a backend response, ready to relay
fn take_cookies(response: &reqwest::Response) -> Vec<(HeaderName, HeaderValue)> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| (header::SET_COOKIE.clone(), value.clone()))
        .collect()
}

fn attach_cookies(response: &mut Response, cookies: Vec<(HeaderName, HeaderValue)>) {
    for (name, value) in cookies {
        response.headers_mut().append(name, value);
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
fn hackclub_login_url(headers: &HeaderMap, return_to: &str) -> String {
    let target = format!(
        "{}/auth/hackclub?returnTo={}",
        origin_for(headers),
        encode_component(return_to)
    );
    format!("/api/v1/auth/login?target={}", encode_component(&target))
}

/// Sanitize a user-supplied `returnTo` so it can never be an open redirect.
fn safe_return_to(return_to: Option<String>) -> String {
    match return_to {
        Some(value) if value.starts_with('/') && !value.contains(':') => value,
        _ => "/dash".to_string(),
    }
}

/// Bail out to `/login` when the incoming cookies don't describe a session.
pub async fn require_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::frontend::models::AuthenticatedUser, Redirect> {
    let cookie = headers.get(header::COOKIE);
    match api::current_user(&state.http_client, &state.config.api_proxy_target, cookie).await {
        Ok(Some(user)) => Ok(user),
        _ => Err(Redirect::to("/login")),
    }
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

pub async fn home() -> HomeTemplate {
    HomeTemplate
}

pub async fn auth_redirect() -> Redirect {
    Redirect::to("/login")
}

// --- Login ---

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn login_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let cookie = headers.get(header::COOKIE);
    if api::is_authenticated(&state.http_client, &state.config.api_proxy_target, cookie).await {
        return Redirect::to("/dash").into_response();
    }
    LoginTemplate::new(String::new(), hackclub_login_url(&headers, "/dash")).into_response()
}

pub async fn login_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    let login_url = api::api_url(&state.config.api_proxy_target, "/auth/login");
    let template = LoginTemplate::new(form.email.clone(), hackclub_login_url(&headers, "/dash"));

    match state.http_client.post(login_url).json(&form).send().await {
        Ok(backend) if backend.status().is_success() => {
            let cookies = take_cookies(&backend);
            let mut response = Redirect::to("/dash").into_response();
            attach_cookies(&mut response, cookies);
            response
        }
        Ok(backend) => {
            let code = backend.text().await.unwrap_or_default();
            if code.trim().is_empty() {
                template
                    .with_error("Something went wrong. Please try again.")
                    .into_response()
            } else {
                template.with_error(code.trim()).into_response()
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "login upstream error");
            template
                .with_error("Something went wrong. Please try again.")
                .into_response()
        }
    }
}

// --- Register ---

#[derive(Debug, Deserialize, Serialize)]
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
    Form(form): Form<RegisterForm>,
) -> Response {
    let register_url = api::api_url(&state.config.api_proxy_target, "/auth/register");

    match state
        .http_client
        .post(register_url)
        .json(&form)
        .send()
        .await
    {
        Ok(backend) if backend.status().is_success() => {
            let cookies = take_cookies(&backend);
            let mut response = Redirect::to("/dash").into_response();
            attach_cookies(&mut response, cookies);
            response
        }
        Ok(backend) => {
            let code = backend.text().await.unwrap_or_default();
            let error = if code.trim().is_empty() {
                "Something went wrong. Please try again.".to_string()
            } else {
                api::friendly_error(code.trim())
            };
            RegisterTemplate {
                error: Some(error),
                first_name: form.first_name,
                last_name: form.last_name,
                email: form.email,
            }
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "register upstream error");
            RegisterTemplate {
                error: Some("Something went wrong. Please try again.".to_string()),
                first_name: form.first_name,
                last_name: form.last_name,
                email: form.email,
            }
            .into_response()
        }
    }
}

// --- Forgot password ---

#[derive(Debug, Deserialize, Serialize)]
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
    let url = api::api_url(&state.config.api_proxy_target, "/auth/forgot-password");

    match state.http_client.post(url).json(&form).send().await {
        Ok(backend) if backend.status().is_success() => ForgotPasswordTemplate {
            error: None,
            message: Some(
                "If an account exists for that email, a reset link has been sent.".to_string(),
            ),
            email: String::new(),
        }
        .into_response(),
        Ok(backend) => {
            let code = backend.text().await.unwrap_or_default();
            let error = if code.trim().is_empty() {
                "Something went wrong. Please try again.".to_string()
            } else {
                api::friendly_error(code.trim())
            };
            ForgotPasswordTemplate {
                error: Some(error),
                message: None,
                email: form.email,
            }
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "forgot password upstream error");
            ForgotPasswordTemplate {
                error: Some("Something went wrong. Please try again.".to_string()),
                message: None,
                email: form.email,
            }
            .into_response()
        }
    }
}

// --- Reset password ---

#[derive(Debug, Deserialize, Serialize)]
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

    let url = api::api_url(&state.config.api_proxy_target, "/auth/reset-password");
    let payload = serde_json::json!({ "token": query.token, "password": form.password });

    match state.http_client.post(url).json(&payload).send().await {
        Ok(backend) if backend.status().is_success() => Redirect::to("/login").into_response(),
        Ok(backend) => {
            let code = backend.text().await.unwrap_or_default();
            let error = if code.trim().is_empty() {
                "Something went wrong. Please try again.".to_string()
            } else {
                api::friendly_error(code.trim())
            };
            ResetPasswordTemplate {
                error: Some(error),
                ..template
            }
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "reset password upstream error");
            ResetPasswordTemplate {
                error: Some("Something went wrong. Please try again.".to_string()),
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
    let cookie = headers.get(header::COOKIE);
    let return_to = safe_return_to(params.return_to);
    if api::is_authenticated(&state.http_client, &state.config.api_proxy_target, cookie).await {
        return Redirect::to(&return_to).into_response();
    }
    Redirect::to("/login").into_response()
}

// --- Logout ---

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let cookie = headers.get(header::COOKIE).cloned();
    let mut response = Redirect::to("/").into_response();

    let url = api::api_url(&state.config.api_proxy_target, "/auth/logout");
    let mut request = state.http_client.post(url);
    if let Some(cookie) = &cookie {
        request = request.header(header::COOKIE, cookie);
    }

    match request.send().await {
        Ok(backend) => attach_cookies(&mut response, take_cookies(&backend)),
        Err(e) => tracing::error!(error = %e, "logout upstream error"),
    }

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
