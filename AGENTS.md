# Agent Instructions

Use this file as the default guide for AI agents working in the repository.

## Brand

- All brand assets and rules live in `docs/brand/`.
- Always use the logo files in `docs/brand/logos/`. Never retype "hackflare", redraw the fire h, or recreate the logo in CSS.
- Served copies of the logos are vendored into `hackflare-app/static/brand/logos/`; reference those from templates as `/static/brand/logos/...`.
- Only use the colors in `docs/brand/tokens/`. Never put white text on Flame orange (#F2611D).
- Font: Instrument Sans for UI text. Use the system monospace stack for code so
  pages do not download an additional code font.
- `DOMAIN` accepts a full `http` or `https` URL or a bare hostname. Bare
  hostnames default to HTTPS and explicit ports are preserved.
- `CDN_URL` is optional. When set, rendered `/static/` asset URLs are rewritten
  to the CDN and the CDN origin is included in CORS. Configure the CDN to serve
  `/static/*` with `Access-Control-Allow-Origin` for the application origin.

## Human In The Loop

- Keep the user informed when making substantial changes.
- Do not commit, push, deploy, or change shared infrastructure without explicit approval.
- Do not modify unrelated user changes in the worktree.

## Quick Rules

- Inspect the source before changing behavior.
- No dead code is allowed. Remove it or comment it out. 
  - For example, `#[allow(dead_code)]` is not allowed.
- Keep changes minimal and localized.
- Follow existing code style and conventions.
- Do not rely on stale documentation when the source disagrees.
- Do not divert from the active task without being asked.
- Do not use em dashes in code, comments, or documentation.
  - Use singular hyphens instead.
  - In the case of double em dashes, use triple hyphens instead.
  - If you are editing documentation that already has em dashes, replace them with hyphens.

## Commands

Run from the repository root:

- `just fmt` - format Rust code
- `just fmt-check` - verify Rust formatting
- `just build-app` - build the merged application
- `just test-app` - test the merged application using embedded PostgreSQL when needed
- `just test-dns` - test the DNS library
- `just lint-app` - lint the merged application
- `just check` - run formatting, linting, and tests
- `just db-up` - start PostgreSQL for local application development
- `just db-down` - stop the development PostgreSQL service
- `just docker-build-app` - build the merged application image

## Repository Layout

- `hackflare-app/` - merged HTTP API and Rust SSR frontend
- `hackflare_dns/` - DNS server library
- `docs/` - project documentation

Read the relevant crate's source before relying on documentation for implementation details.

## Frontend Conventions

- The frontend uses Axum and Askama templates.
- Keep each dashboard page in its own template.
- Put presentation styles in `static/app.css`; never add inline `style` attributes.
- Use the vendored Lucide assets in `static/icons` for dashboard icons. You can add more if neccesary.
- Pages without an implementation must say `Coming Soon`.
- Run the frontend build and clippy checks after frontend changes.

## Code Conventions

- Keep route behavior, configuration, middleware, and persistence logic near the module that owns it.
- Keep CDN URL rewriting and CDN-specific behavior in `hackflare-app/src/frontend/cdn.rs`.
- Keep CORS origin configuration in `hackflare-app/src/frontend/cors.rs`.
- Avoid blocking I/O in async code.
- Prefer explicit error handling over panics.
- Add comments only when they clarify non-obvious behavior.


## Finally

Thanks for your contributions <3, Love the support.
