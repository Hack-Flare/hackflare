//! Public content pages: the team page and the sitemap.

use askama::Template;
use axum::{extract::State, http::HeaderMap, response::{IntoResponse, Response}};

use crate::{frontend::{pages::{SitemapTemplate, TeamTemplate}, routes::handlers::{origin_for, public_header}}, state::AppState};

/// `GET /ourteam`.
pub async fn team(State(state): State<AppState>, headers: HeaderMap) -> TeamTemplate {
    TeamTemplate {
        header: public_header(&state, &headers).await,
    }
}

/// `GET /sitemap.xml`.
pub async fn sitemap(headers: HeaderMap) -> Response {
    let template = SitemapTemplate { origin: origin_for(&headers) };
    match template.render() {
        Ok(xml) => (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/xml")],
            xml,
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "failed to render sitemap");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                "INTERNAL SERVER ERROR",
            )
                .into_response()
        }
    }
}
