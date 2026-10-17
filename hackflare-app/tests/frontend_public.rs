use hackflare_app::frontend::routes::public::DOCS;
use hackflare_app::frontend::routes::public::*;

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
