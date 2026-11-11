# Hackflare interface design system

This document covers layout and hierarchy craft for the dashboard.
It is not a second brand guide: [docs/brand/](brand/README.md) is the source of truth for color, type, logos, and buttons.
Where this file and the brand disagree, the brand wins.

## Brand bindings

These rules are already wired into `hackflare-app/static/app.css`.
Follow them instead of re-deciding them per page.

- Palette: Flame `#F2611D`, Ember `#FFC24A`, Ink `#16140F`, Paper `#FAF9F5`, Char `#B8400C`, Smoke `#57534A`, Ash `#ECEBE6`.
- `base.html` loads `/static/brand/tokens/tokens.css`, and `app.css` maps every semantic token (`--background`, `--card`, and the rest) onto the `--hf-*` variables.
  Never write a raw hex or oklch color into `app.css`; add or change a mapping instead.
- Never put white text on Flame.
  `.btn-orange` is Flame plus Ink.
  The `.cta` panel is flat Flame with Ink text and a `.btn-on-flame` action (Ink plus Paper).
- `--link` is the orange that is safe as text: Char on light, Flame on dark.
  Flame is never body text on Paper.
- Status colors stay in palette.
  `--success` is Ink on light and Flame on dark.
  `--warning` and `--destructive` are Char on light and Ember on dark.
- Type: Instrument Sans for UI text, self-hosted from `static/fonts/instrument-sans-latin.woff2` and declared with `@font-face` in `static/brand/tokens/tokens.css`.
  Code uses the system monospace stack (`--font-mono`, bound to `code`, `kbd`, `pre`, and `samp`).
  Body is 17px/1.55.
- Radii come from the brand tokens: `--hf-radius-button` 10px, `--hf-radius-card` 16px, `--hf-radius-panel` 20px.
  Do not add new radius values.
- The logo is artwork only, served from `hackflare-app/static/brand/logos/`.
  The wordmark is never set in a font, so there is no `<span>Hackflare</span>` beside the mark.
  Light and dark variants swap via `.brand-logo-light` and `.brand-logo-dark`.
- The product name in copy is "Hackflare", not "HackFlare".

## Direction and feel

Hackflare is a DNS and edge-network platform, Cloudflare-shaped but scrappy and technical.
It leans on its own domain (nameservers, zones, propagation, edge) rather than a stock admin-dashboard template.
Flame is the one accent and is reserved for meaning: primary actions, the empty-state icon tint, and signature elements.
It is never decoration.

## Depth and spacing

- Depth is borders only plus a single subtle card shadow.
  Cards are Ash on Paper, so color separation does most of the work.
  Do not add layered shadows or surface-tint elevation.
- Spacing base unit is 4px, using the existing `.gap-*` and `.p-*` utilities.
- Radius uses the brand tokens only.

## Hierarchy decisions

- **Stat tiles**: use `.tools-row` with `.tool-tile` and `.tool-meta` for the label, and `.admin-stat` for the value.
  The label is demoted and the value is the hero (26px, 700 weight, tabular-nums, tight tracking).
  Do not re-add a plain small-label-next-to-large-value pattern per page.
- **Technical and tabular data**: DNS record names and values, domain names, timestamps, durations, and status codes render in `<code>` inside table cells.
  `.table td code` applies monospace and tabular-nums.
  Do not leave a DNS/network-technical value in the body font.
- **Tables**: any `<table class="table">` gets row hover (`.table tbody tr:hover` to `var(--muted)`) and a top border between rows for free.
  No per-table CSS is needed.

## Empty and unbuilt states

- **Empty state**: `.empty-state` plus `.empty-state-icon` plus `.badge-soon`.
  The icon sits in a tinted accent circle (48px, 12px radius) with a heading and a short domain-specific description above a small "Coming Soon" pill.
  Used by `placeholder.html`, `traffic.html`, `logs.html`, `notifications.html`, and `help.html`.
- **Feature teaser**: `/dash/soon/{tool}` renders `dash/soon.html`, which uses `.soon`, `.soon-icon`, and `.soon-badge`.
- Do not revert to a bare `<p class="text-muted">Coming Soon</p>`.
  The literal words "Coming Soon" belong in the `.badge-soon` pill, with a real heading and description above it that names what the feature will do.
  Keep that copy tied to the DNS and edge domain, and to two sentences at most.

## Working with templates

- Templates are compiled into the binary by Askama at build time.
  A running `cargo run` process does not pick up template edits, only `static/*` changes (CSS, icons, and JS are read from disk).
  Rebuild and restart the dev server to see a template change.
- `dash/base.html` dispatches on the `active` field to include the right content template.
  No per-route icon or description is passed from Rust; each template hardcodes its own copy.
  Keep it that way unless a page actually needs dynamic content.
- Use the vendored Lucide icons in `static/icons`.
  Match the existing stroke-based format (`stroke-width="2"`, `stroke-linecap="round"`, and the rest).
  `static/icons/globe.svg` is a good reference.
