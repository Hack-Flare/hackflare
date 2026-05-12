use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::Validation;
use reqwest::StatusCode;

use crate::{routes::auth::JwtClaims, state::AppState};

#[derive(Clone)]
pub(crate) struct CurrentUser {
    pub(crate) claims: JwtClaims,
}

pub(crate) async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let jwt = jar
        .get("jwt")
        .map(|c| c.value().to_owned())
        .ok_or((StatusCode::UNAUTHORIZED, "missing_jwt"))?;

    let claims = jsonwebtoken::decode::<JwtClaims>(
        &jwt,
        &state.config.jwt_decoding_key,
        &Validation::default(),
    )
    .map_err(|error| {
        debug!(%error, "jwt validation failed");
        (StatusCode::UNAUTHORIZED, "invalid_jwt")
    })?
    .claims;

    let user = CurrentUser { claims };

    // TODO: get user data from DB

    req.extensions_mut().insert(user);

    debug!("user was authorized!");
    Ok(next.run(req).await)
}
