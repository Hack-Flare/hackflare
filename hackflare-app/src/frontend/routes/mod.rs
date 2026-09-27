//! Route wiring for the frontend server

use axum::{
    body::{Body, to_bytes},
    http::{Request, header},
    middleware::{self, Next},
    response::Response,
    Router,
    routing::get,
};
use axum_extra::extract::CookieJar;
use tower_http::services::ServeDir;

pub mod dash;
pub mod handlers;
pub mod public;

use crate::state::AppState;

async fn apply_theme(request: Request<Body>, next: Next) -> Response {
    let dark = CookieJar::from_headers(request.headers())
        .get("theme")
        .is_some_and(|cookie| cookie.value() == "dark");
    let mut response = next.run(request).await;

    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    if !is_html {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match to_bytes(body, 4 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::error!(%error, "failed to read rendered HTML response");
            return Response::from_parts(parts, Body::empty());
        }
    };
    let Ok(html) = String::from_utf8(bytes.to_vec()) else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    let class = if dark { " class=\"dark\"" } else { "" };
    let html = html.replacen("<html lang=\"en\">", &format!("<html lang=\"en\"{class}>") , 1);
    parts.headers.remove(header::CONTENT_LENGTH);
    response = Response::from_parts(parts, Body::from(html));
    response
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::home))
        .route("/auth", get(handlers::auth_redirect))
        .route(
            "/login",
            get(handlers::login_get).post(handlers::login_post),
        )
        .route(
            "/register",
            get(handlers::register_get).post(handlers::register_post),
        )
        .route(
            "/forgot-password",
            get(handlers::forgot_get).post(handlers::forgot_post),
        )
        .route(
            "/reset-password",
            get(handlers::reset_get).post(handlers::reset_post),
        )
        .route("/auth/hackclub", get(handlers::hackclub))
        .route("/theme", get(handlers::theme))
        .route("/logout", get(handlers::logout))
        .route("/health", get(handlers::health))
        .route("/docs", get(public::docs))
        .route("/ourteam", get(public::team))
        .route("/dash", get(dash::index))
        .route("/dash/domains", get(dash::domains_get).post(dash::domains_post))
        .route("/dash/settings", get(dash::settings_get).post(dash::settings_post))
        .route("/dash/tokens", get(dash::tokens_get).post(dash::tokens_post))
        .route("/dash/soon/{tool}", get(dash::soon))
        .route(
            "/dash/notifications",
            get(dash::notifications_get).post(dash::notifications_post),
        )
        .route("/dash/{section}", get(dash::section))
        .route(
            "/dash/domains/{domain}/dns",
            get(dash::dns_get).post(dash::dns_post),
        )
        .route("/dash/domains/{domain}/{sub}", get(dash::domain_sub))
        .nest_service("/static", ServeDir::new(state.config.static_dir.clone()))
        .fallback(handlers::not_found)
        .layer(middleware::from_fn(apply_theme))
        .with_state(state)
}
