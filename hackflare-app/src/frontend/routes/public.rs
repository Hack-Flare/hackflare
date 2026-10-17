//! Public content pages: documentation and the team page.

use axum::{extract::{Query, State}, http::HeaderMap, response::{IntoResponse, Response}};
use pulldown_cmark::{Event, Options, Parser, html};
use serde::Deserialize;

use crate::{frontend::{pages::{DocLink, DocsTemplate, TeamTemplate}, routes::handlers::public_header}, state::AppState};

use super::handlers::not_found;

/// (slug, markdown) in sidebar order. Embedded at compile time so the docs
/// ship inside the binary.
pub const DOCS: &[(&str, &str)] = &[
    ("getting-started", include_str!("../../../docs/getting-started.md")),
    ("managing-domains", include_str!("../../../docs/managing-domains.md")),
    ("dns-records", include_str!("../../../docs/dns-records.md")),
    ("faq", include_str!("../../../docs/faq.md")),
];

/// A doc's title is its first `# ` heading, falling back to the slug.
pub fn doc_title<'a>(slug: &'a str, markdown: &'a str) -> &'a str {
    markdown
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .unwrap_or(slug)
}

/// Render markdown to HTML. Raw HTML in the source is escaped rather than
/// passed through, so the output is always safe to embed.
pub fn render_markdown(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::ENABLE_TABLES).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

#[derive(Debug, Deserialize)]
pub struct DocsQuery {
    doc: Option<String>,
}

/// `GET /docs?doc=<slug>`, defaulting to the first doc.
pub async fn docs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DocsQuery>,
) -> Response {
    let slug = query.doc.as_deref().unwrap_or(DOCS[0].0);
    let Some((slug, markdown)) = DOCS.iter().find(|(s, _)| *s == slug) else {
        return not_found().await;
    };

    DocsTemplate {
        header: public_header(&state, &headers).await,
        title: doc_title(slug, markdown).to_string(),
        content: render_markdown(markdown),
        links: DOCS
            .iter()
            .map(|(s, md)| DocLink {
                slug: s.to_string(),
                title: doc_title(s, md).to_string(),
                active: s == slug,
            })
            .collect(),
    }
    .into_response()
}

/// `GET /ourteam`.
pub async fn team(State(state): State<AppState>, headers: HeaderMap) -> TeamTemplate {
    TeamTemplate {
        header: public_header(&state, &headers).await,
    }
}
