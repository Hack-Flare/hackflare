# Agent Instructions

## Layout

- `hackflare-app/` - merged HTTP API and Rust SSR frontend.
- `hackflare-dns/` - DNS server library.
- `hackflare-bench/` - benchmarking tool.
- `database/` - SQL migrations.
- `deployment/` - Docker Compose deployment files.
- `docs/API.md` - HTTP API reference.
- `docs/DESIGN.md` - dashboard design system.
- `docs/brand/` - brand kit: logos and design tokens.

## Commands

Run from the repository root:

- `just fmt` - format Rust code.
- `just fmt-check` - verify Rust formatting.
- `just lint-app` - lint the merged application.
- `just test-app` - test the merged application.
- `just test-dns` - test the DNS library.
- `just check` - run formatting, linting, and tests.
- `just build-app` - build the merged application.

## Rules

- Read the source before changing behavior.
- When documentation and source disagree, trust the source.
- Keep changes minimal and localized.
- Follow existing code style and conventions.
- Do not divert from the active task without being asked.
- Avoid blocking I/O in async code.
- Prefer explicit error handling over panics.
- No dead code.
  - Remove it or comment it out.
  - Never add `#[allow(dead_code)]`.

## Writing

These rules apply to code, comments, and documentation.

- One sentence per line.
  - Never wrap a sentence across lines.
- No em dashes.
  - Use a single hyphen instead, or a triple hyphen for a double em dash.
  - Replace em dashes with hyphens when editing text that already has them.
- Add comments only when they clarify non-obvious behavior.

## Human In The Loop

- Keep the user informed when making substantial changes.
- Do not modify unrelated user changes in the worktree.
- Do not commit, push, deploy, or change shared infrastructure without explicit approval.

## Frontend

- The frontend is Axum plus Askama templates.
- Keep each dashboard page in its own template.
- Put presentation styles in `hackflare-app/static/app.css`; never add inline `style` attributes.
- Use the vendored Lucide assets in `hackflare-app/static/icons` for dashboard icons; add more when needed.
- Pages without an implementation must say `Coming Soon`, or better, not exist at all.
- Run `just lint-app` after frontend changes; it compiles the templates too.

## Brand

- All brand assets and rules live in `docs/brand/`.
- Use the logo files in `docs/brand/logos/`; never retype "hackflare", redraw the fire h, or recreate the logo in CSS.
- Templates reference the vendored copies in `hackflare-app/static/brand/` as `/static/brand/...`; re-vendor after changing a brand file.
- Only use the colors in `docs/brand/tokens/`.
- Never put white text on Flame orange (`#F2611D`).
- Instrument Sans for UI text; the system monospace stack for code.
