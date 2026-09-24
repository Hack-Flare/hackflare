use std::net::{IpAddr, Ipv4Addr};

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use jsonwebtoken::Validation;
use axum::http::{HeaderMap, header};
use reqwest::StatusCode;

use crate::{
    api::models::{CurrentUser, JwtClaims, db::UserSession},
    state::AppState,
};

fn virtual_session_for_api_key(api_key: &crate::api::services::api_keys::ApiKey) -> UserSession {
    UserSession {
        id: api_key.id,
        user_id: api_key.user_id.clone(),
        ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        expires_at: Utc::now() + chrono::Duration::days(365),
        created_at: api_key.created_at,
        revoked_at: None,
    }
}

pub(crate) async fn auth_middleware(
    State(app_state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let config = &app_state.config;
    let users = &app_state.users;
    let user_sessions = &app_state.user_sessions;
    let api_keys = &app_state.api_keys;

    // Try Bearer token (API key) first
    if let Some(auth_header) = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        && let Some(raw_key) = auth_header.strip_prefix("Bearer ")
    {
        let key = api_keys.find_by_key(raw_key).await.map_err(|error| {
            error!(%error, "failed to lookup api key");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

        if let Some(api_key) = key {
            let user = users.get_by_id(&api_key.user_id).await.map_err(|error| {
                error!(%error, "failed to get user");
                (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
            })?;

            let Some(user) = user else {
                warn!("api key valid but no user exists");
                return Err((StatusCode::UNAUTHORIZED, "unauthorized"));
            };

            // Update last_used_at in background
            let _ = api_keys.update_last_used(api_key.id).await;

            let session = virtual_session_for_api_key(&api_key);
            let user = CurrentUser { session, user };
            req.extensions_mut().insert(user);
            debug!("api key auth succeeded");
            return Ok(next.run(req).await);
        }
    }

    // Fall back to JWT cookie
    let jwt = jar
        .get("jwt")
        .map(|c| c.value().to_owned())
        .ok_or((StatusCode::UNAUTHORIZED, "missing_jwt"))?;

    let claims =
        jsonwebtoken::decode::<JwtClaims>(&jwt, &config.jwt_decoding_key, &Validation::default())
            .map_err(|error| {
                debug!(%error, "jwt validation failed");
                (StatusCode::UNAUTHORIZED, "invalid_jwt")
            })?
            .claims;

    if claims.typ.as_deref() == Some("refresh") {
        warn!("refresh token used as access token");
        return Err((StatusCode::UNAUTHORIZED, "invalid_token_type"));
    }

    let user = users.get_by_id(&claims.sub).await.map_err(|error| {
        error!(%error, "failed to get user");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    let Some(user) = user else {
        warn!("jwt found but no user exists");
        return Err((StatusCode::UNAUTHORIZED, "unauthorized"));
    };
    debug!(user.id, "got user");

    let session = user_sessions
        .get_by_id(&claims.jit)
        .await
        .map_err(|error| {
            error!(%error, "failed to get user session");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    let Some(session) = session else {
        debug!(user.id, %claims.jit, "no session found");
        return Err((StatusCode::UNAUTHORIZED, "unauthorized"));
    };
    debug!(user.id, "user authenticated");

    let user = CurrentUser { session, user };

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

pub(crate) async fn user_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Option<crate::api::models::db::User> {
    let jwt = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;
                (name == "jwt").then_some(value)
            })
        })?;
    let claims = jsonwebtoken::decode::<JwtClaims>(
        jwt,
        &state.config.jwt_decoding_key,
        &Validation::default(),
    )
    .ok()?
    .claims;
    if claims.typ.as_deref() == Some("refresh") {
        return None;
    }
    state.users.get_by_id(&claims.sub).await.ok().flatten()
}
