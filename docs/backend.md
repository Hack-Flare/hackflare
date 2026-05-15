# Hackflare Backend (hackflare_api)

Overview
--------

The backend implements a small HTTP API used by the frontend. It currently focuses on authentication via Hack Club Auth (HCA) and a minimal user endpoint.

Key files
- `hackflare_api/src/main.rs` — binary entrypoint and server bootstrap
- `hackflare_api/src/config.rs` — environment-driven configuration
- `hackflare_api/src/state.rs` — shared `AppState` (http client, config)
- `hackflare_api/src/routes/mod.rs` — route assembly (v1 namespace)
- `hackflare_api/src/routes/auth.rs` — OAuth login, callback, logout
- `hackflare_api/src/routes/users.rs` — user endpoints
- `hackflare_api/src/middlewares/auth.rs` — `jwt` cookie validation middleware

HTTP surface
- `GET /api/v1/auth/login` — start OAuth flow (redirect to HCA)
- `GET /api/v1/auth/callback` — OAuth callback; exchanges code, fetches identity, issues `jwt` cookie
- `POST /api/v1/auth/logout` — clears `jwt` cookie
- `GET /api/v1/users/me` — returns current user id (requires `jwt` cookie)

Authentication & Sessions
- OAuth via Hack Club Auth; `login` persists CSRF and optional `target` in an in-memory session
- Sessions use `tower_sessions::MemoryStore` and expire after 15 minutes inactivity
- JWTs are signed with the base64 secret from `API_JWT_SECRET` and set as `HttpOnly` cookie for 24 hours

Configuration
- Required env vars:
  - `API_HCA_REDIRECT_URI` (must use `http` or `https`)
  - `API_JWT_SECRET` (base64-encoded secret used to sign JWTs)
  - `API_HCA_CLIENT_ID`
  - `API_HCA_CLIENT_SECRET`
- Optional:
  - `API_BIND_ADDR` (default `0.0.0.0:8080`)

Running locally
- Build: `cargo build -p hackflare-api`
- Run: `cargo run -p hackflare-api`
- Tests: `cargo test -p hackflare-api`
- Dev compose (repo): `docker compose -f compose.dev.yml --profile backend up -d`

Notes & TODO
- No persistent user DB yet — the code contains TODO comments to upsert users
- No health or admin endpoints are implemented
- No DNS management endpoints are present in the backend; DNS functionality is provided by the `hackflare_dns` crate
