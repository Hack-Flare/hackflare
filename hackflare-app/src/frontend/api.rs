//! Server-side client for the backend API.

use axum::http::{HeaderValue, header::COOKIE};
use reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};

use crate::frontend::models::AuthenticatedUser;

const ERROR_MESSAGES: &[(&str, &str)] = &[
    ("invalid_email_or_password", "Invalid email or password."),
    (
        "email_already_registered",
        "An account with this email already exists.",
    ),
    (
        "password_too_short",
        "Password must be at least 8 characters.",
    ),
    ("invalid_email", "Please enter a valid email address."),
    (
        "email_and_password_required",
        "Email and password are required.",
    ),
    (
        "invalid_or_expired_token",
        "This reset link is invalid or has expired.",
    ),
    ("email_required", "Please enter your email address."),
    ("token_required", "Reset token is required."),
    (
        "current_password_incorrect",
        "Your current password is incorrect.",
    ),
];

/// Map a backend error code to a user-facing message (ported from `api.ts`).
pub fn friendly_error(error: &str) -> String {
    ERROR_MESSAGES
        .iter()
        .find(|(code, _)| *code == error)
        .map(|(_, message)| (*message).to_string())
        .unwrap_or_else(|| error.to_string())
}

/// Build a backend API URL for the given `/api/v1`-relative path.
pub fn api_url(proxy_target: &str, path: &str) -> String {
    format!("{}/api/v1{}", proxy_target.trim_end_matches('/'), path)
}

fn with_cookie(
    request: reqwest::RequestBuilder,
    cookie: Option<&HeaderValue>,
) -> reqwest::RequestBuilder {
    if let Some(value) = cookie {
        request.header(COOKIE, value)
    } else {
        request
    }
}

async fn ensure_success(response: reqwest::Response) -> anyhow::Result<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let message = if body.trim().is_empty() {
        status.to_string()
    } else {
        body
    };
    anyhow::bail!("{}: {}", status, friendly_error(message.trim()))
}

pub async fn get_json<T: DeserializeOwned>(
    client: &reqwest::Client,
    proxy_target: &str,
    path: &str,
    cookie: Option<&HeaderValue>,
) -> anyhow::Result<T> {
    let response = with_cookie(client.get(api_url(proxy_target, path)), cookie)
        .send()
        .await?;
    ensure_success(response)
        .await?
        .json::<T>()
        .await
        .map_err(Into::into)
}

pub async fn send_json<B: Serialize>(
    client: &reqwest::Client,
    proxy_target: &str,
    method: Method,
    path: &str,
    body: &B,
    cookie: Option<&HeaderValue>,
) -> anyhow::Result<()> {
    let response = with_cookie(
        client
            .request(method, api_url(proxy_target, path))
            .json(body),
        cookie,
    )
    .send()
    .await?;
    ensure_success(response).await.map(|_| ())
}

pub async fn send_json_response<T: DeserializeOwned, B: Serialize>(
    client: &reqwest::Client,
    proxy_target: &str,
    method: Method,
    path: &str,
    body: &B,
    cookie: Option<&HeaderValue>,
) -> anyhow::Result<T> {
    let response = with_cookie(
        client
            .request(method, api_url(proxy_target, path))
            .json(body),
        cookie,
    )
    .send()
    .await?;
    ensure_success(response)
        .await?
        .json::<T>()
        .await
        .map_err(Into::into)
}

pub async fn send_empty(
    client: &reqwest::Client,
    proxy_target: &str,
    method: Method,
    path: &str,
    cookie: Option<&HeaderValue>,
) -> anyhow::Result<()> {
    let response = with_cookie(client.request(method, api_url(proxy_target, path)), cookie)
        .send()
        .await?;
    ensure_success(response).await.map(|_| ())
}

/// Fetch the authenticated user for a request, if any.
///
/// Returns `Ok(None)` for any non-2xx response (e.g. no/invalid session).
pub async fn current_user(
    client: &reqwest::Client,
    proxy_target: &str,
    cookie: Option<&HeaderValue>,
) -> anyhow::Result<Option<AuthenticatedUser>> {
    let mut request = client.get(api_url(proxy_target, "/users/me"));
    if let Some(cookie) = cookie {
        request = request.header(COOKIE, cookie);
    }

    let response = request.send().await?;
    if !response.status().is_success() {
        return Ok(None);
    }

    let user = response.json::<AuthenticatedUser>().await?;
    Ok(Some(user))
}

/// Whether the user is authenticated, based on the incoming cookie header.
pub async fn is_authenticated(
    client: &reqwest::Client,
    proxy_target: &str,
    cookie: Option<&HeaderValue>,
) -> bool {
    current_user(client, proxy_target, cookie)
        .await
        .is_ok_and(|user| user.is_some())
}
