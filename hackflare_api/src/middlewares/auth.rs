use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::Validation;
use reqwest::StatusCode;

use crate::{
    config::Config,
    models::{CurrentUser, JwtClaims},
    services::{user_sessions::UserSessionsService, users::UsersService},
};

pub(crate) async fn auth_middleware(
    State(config): State<Arc<Config>>,
    State(users): State<UsersService>,
    State(user_sessions): State<UserSessionsService>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
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
