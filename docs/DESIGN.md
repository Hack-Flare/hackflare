# Hackflare Interface Design System

> **The brand kit outranks this file.** `brand/` and
> `.claude/skills/hackflare-brand/SKILL.md` are the source of truth for color,
> type, logo and button rules. Where this document and the brand disagree, the
> brand wins. What stays useful here is the layout and hierarchy craft: stat
> hierarchy, empty states, monospace for technical values, table states.

## Brand bindings (applied)

- Palette: Flame `#F2611D`, Ember `#FFC24A`, Ink `#16140F`, Paper `#FAF9F5`,
  Char `#B8400C`, Smoke `#57534A`, Ash `#ECEBE6`. `app.css` imports
  `/static/brand/tokens/tokens.css` and maps every semantic token
  (`--background`, `--card`, and the rest) onto those `--hf-*` variables. Never
  write a raw hex or oklch color into `app.css` again; add a mapping instead.
- Never white text on Flame. `.btn-orange` is Flame plus Ink; the `.cta` panel
  is flat Flame with Ink text and a `.btn-on-flame` (Ink plus Paper) action.
- `--link` is the orange that is safe as text: Char on light, Flame on dark.
  Flame is never body text on Paper (3.1:1).
- Status colors stay in palette: `--success` is Ink on light and Flame on dark,
  `--warning` and `--destructive` are Char on light and Ember on dark.
- Type: Instrument Sans everywhere, JetBrains Mono for code only (`--font-mono`
  is bound to `code, kbd, pre, samp`). Body is 17px/1.55, loaded from a Google
  Fonts `<link>` in `base.html`.
- Radii come from the brand: `--hf-radius-button` 10px, `--hf-radius-card`
  16px, `--hf-radius-panel` 20px. No more `calc(var(--radius) * n)`.
- Logo is artwork only, from `hackflare-app/static/brand/logos/`. The wordmark
  is never set in a font, so no `<span>Hackflare</span>` beside the mark. Light
  and dark variants swap via `.brand-logo-light` / `.brand-logo-dark`.
- Product name in copy is "Hackflare", not "HackFlare".

## Direction and feel

DNS/edge-network platform (Cloudflare-shaped), Hackflare branded: scrappy,
technical, built by teens for builders. Not corporate-SaaS-generic; leans on
its own domain (nameservers, zones, propagation, edge) rather than a stock
admin-dashboard template. Flame is the one accent and is reserved for meaning:
primary actions, the empty-state icon tint, and signature elements, never
decoration.

## Depth strategy and spacing

- Depth: borders-only + single subtle card shadow. Cards are Ash on Paper, so color separation does most of the work.
  Don't add layered shadows or surface-tint elevation - stay consistent with
  what's already in `static/app.css`.
- Spacing base unit: 4px, existing scale in `.gap-*`/`.p-*` utilities.
- Radius: use the brand tokens only, `--hf-radius-button` / `--hf-radius-card` /
  `--hf-radius-panel`. Do not introduce new radius values.

## Hierarchy decisions

- **Stat cards**: label is demoted (11px / 700 weight / uppercase / tracked /
  muted), value is the hero (2rem / 700 weight / tabular-nums / tight
  tracking). This is the default `.stat-label` / `.stat-value` styling now -
  don't re-add a plain 14px-label-next-to-30px-value pattern per page.
- **Technical/tabular data** (DNS record name/value/TTL, domain names,
  timestamps, durations, status codes) renders in `<code>` inside table
  cells - monospace + tabular-nums via `.table td code`. Apply this any time
  a new table shows a DNS/network-technical value; don't leave it in body
  font.
- **Tables**: rows get a hover state (`.table tbody tr:hover` → `var(--muted)`)
  and a top border between rows. Any new `<table class="table">` gets this
  for free - no per-table CSS needed.

## Key component patterns

- **Empty state** (`.empty-state` + `.empty-state-icon` + `.badge-soon`) -
  icon in a tinted accent circle (48px, 12px radius, Flame at 10% background, full Flame icon color) + `<h3>` + one-line domain-specific
  description + a small uppercase "Coming Soon" pill. Used on every
  unimplemented dashboard page (`firewall.html`, `workers.html`, `tunnel.html`,
  `ssl.html`, `redirects.html`, `performance.html`, `traffic.html`'s chart
  card, `placeholder.html` fallback). **Never revert to a bare
  `<p class="text-muted">Coming Soon</p>`** - CLAUDE.md still requires the
  literal words "Coming Soon" somewhere on the page, but it goes in the
  `.badge-soon` pill, with a real heading/description above it that names
  what the feature will actually do (2 sentences max, tied to the DNS/edge
  domain, not generic "this feature is in development" copy).

## Notes for future work

- New lucide icons added this pass: `sun.svg`, `moon.svg`, `lock.svg`
  (`icon-ssl`), `route.svg` (`icon-redirects`). Follow the same vendored
  header comment / stroke-based format as the existing icons in
  `static/icons/` when adding more - check `static/icons/globe.svg` for the
  exact attribute set (`stroke-width="2"`, `stroke-linecap="round"`, etc).
- Templates are compiled into the binary by Askama at build time - a running
  `cargo run` process will NOT pick up template edits, only `static/*`
  changes (CSS/icons/JS are read from disk). Any template change needs a
  rebuild + restart of the dev server before it's visible in the browser.
- `dash/base.html` dispatches on the `active` field to include the right
  content template - there's no per-route icon/description data passed from
  Rust, each "Coming Soon" template hardcodes its own copy. Keep it that way
  (no need to thread icon/description through `DashboardTemplate`) unless a
  page actually needs dynamic content.
