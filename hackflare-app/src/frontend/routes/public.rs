//! Public content pages: documentation and the team page.

use axum::{extract::Query, response::{IntoResponse, Response}};
use pulldown_cmark::{Event, Options, Parser, html};
use serde::Deserialize;

use crate::frontend::pages::{DocLink, DocsTemplate, TeamTemplate};

use super::handlers::not_found;

/// (slug, markdown) in sidebar order. Embedded at compile time so the docs
/// ship inside the binary.
const DOCS: &[(&str, &str)] = &[
    ("getting-started", include_str!("../../../docs/getting-started.md")),
    ("managing-domains", include_str!("../../../docs/managing-domains.md")),
    ("dns-records", include_str!("../../../docs/dns-records.md")),
    ("faq", include_str!("../../../docs/faq.md")),
];

/// A doc's title is its first `# ` heading, falling back to the slug.
fn doc_title<'a>(slug: &'a str, markdown: &'a str) -> &'a str {
    markdown
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .unwrap_or(slug)
}

/// Render markdown to HTML. Raw HTML in the source is escaped rather than
/// passed through, so the output is always safe to embed.
fn render_markdown(markdown: &str) -> String {
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
pub async fn docs(Query(query): Query<DocsQuery>) -> Response {
    let slug = query.doc.as_deref().unwrap_or(DOCS[0].0);
    let Some((slug, markdown)) = DOCS.iter().find(|(s, _)| *s == slug) else {
        return not_found().await;
    };

    DocsTemplate {
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
pub async fn team() -> TeamTemplate {
    TeamTemplate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_doc_has_a_heading_title() {
        for (slug, markdown) in DOCS {
            assert_ne!(doc_title(slug, markdown), *slug, "{slug} has no # heading");
        }
    }

    #[test]
    fn raw_html_is_escaped() {
        let html = render_markdown("hi <script>alert(1)</script>");
        assert!(!html.contains("<script>"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }
}
