# Hackflare Backend API

This document describes the API served by the merged application in [hackflare-app](../hackflare-app). If this file ever disagrees with the Rust source, trust the source.

## Current Surface Area

The application entrypoint is [hackflare-app/src/main.rs](../hackflare-app/src/main.rs). HTTP API routing is assembled in [hackflare-app/src/api/routes/mod.rs](../hackflare-app/src/api/routes/mod.rs), with auth in [hackflare-app/src/auth/routes.rs](../hackflare-app/src/auth/routes.rs), user routes in [hackflare-app/src/api/routes/users.rs](../hackflare-app/src/api/routes/users.rs), and JWT cookie validation in [hackflare-app/src/auth/middleware.rs](../hackflare-app/src/auth/middleware.rs).

The API is grouped into these route families:

- `/api/v1/auth` - authentication and password management
- `/api/v1/users` - current user information
- `/api/v1/sessions` - session management
- `/api/v1/dns` - DNS zones and records
- `/api/v1/admin` - administrative configuration, users, and statistics
- `/api/v1/settings` - API keys and account settings
- `/api/v1/logs` - query logs
- `/api/v1/notifications` - user notifications
- `/api/v1/traffic` - user and administrative traffic data

## Authentication Flow

Authentication is an OAuth flow against Hack Club Auth.

1. `GET /api/v1/auth/login` creates a short-lived session entry, stores a CSRF token, and redirects the browser to `https://auth.hackclub.com/oauth/authorize`.
2. `GET /api/v1/auth/callback` verifies the returned `code` and `state`, exchanges the code for a Hack Club access token, fetches the user profile from Hack Club, and issues a signed `jwt` cookie.
3. `GET /api/v1/users/me` reads the `jwt` cookie, validates the JWT, and returns the current user id.

## Route Reference

### `GET /api/v1/auth/login`

Starts the Hack Club OAuth redirect.

Query parameters:

- `target` optional. When present, it is saved in the session and used as the post-login redirect target.

Behavior:

- Generates a random CSRF token.
- Stores the CSRF token in an in-memory session store.
- Stores `target` in the session if provided.
- Redirects to Hack Club Auth with the configured `client_id`, `redirect_uri`, `response_type=code`, requested scopes, and the CSRF token in `state`.

Response:

- `302 Found` redirect to `https://auth.hackclub.com/oauth/authorize?...`

### `GET /api/v1/auth/callback`

Finishes the OAuth exchange and creates the application session.

Query parameters:

- `code` required. OAuth authorization code returned by Hack Club.
- `state` required. Must match the CSRF token stored by `/api/v1/auth/login`.

Behavior:

- Reads and removes the CSRF token from the session.
- Rejects the request with `400 missing_auth_state` if the session token is missing.
- Rejects the request with `400 csrf_token_mismatch` if the returned state does not match.
- Exchanges the code for a Hack Club access token.
- Fetches the current user profile from `https://auth.hackclub.com/api/v1/me`.
- Signs a JWT with the user id in `sub`.
- Sets a `jwt` cookie and redirects the browser to the stored target or `/`.

Response behavior:

- Success: `302 Found` with `Set-Cookie: jwt=...` and `Location: <target>`.
- Failure responses are plain text status bodies, not JSON.

Common failure codes:

- `400 exchange_failed`
- `400 hca_rejected_exchange`
- `400 missing_auth_state`
- `400 csrf_token_mismatch`
- `401 hca_identity_denied`
- `500 identity_request_failed`
- `500 invalid_user_data`
- `500 jwt_encode_error`

### `GET /api/v1/users/me`

Returns the authenticated user id.

Authentication:

- Requires the `jwt` cookie.
- The cookie is validated by [hackflare-app/src/auth/middleware.rs](../hackflare-app/src/auth/middleware.rs).

Response:

```json
{ "id": "<user-id>" }
```

Common failure codes:

- `401 missing_jwt`
- `401 invalid_jwt`

### `POST /api/v1/auth/logout`

Clears the `jwt` cookie used by the frontend session.

Response:

- `204 No Content`

This endpoint is used by the frontend sign-out action so the browser session is removed on the backend as well.

## Session And Cookie Details

- The OAuth session state is stored in an in-memory `tower_sessions::MemoryStore`.
- Session entries expire after 15 minutes of inactivity.
- The issued access and refresh cookies are `HttpOnly`, use `SameSite=Lax`, and are marked `Secure` when `HCA_REDIRECT_URI` uses `https`.
- The access and refresh JWT lifetimes default to 15 minutes and 30 days respectively.

## Configuration

The application reads configuration from environment variables in [hackflare-app/src/config.rs](../hackflare-app/src/config.rs).

Required variables:

- `HCA_REDIRECT_URI`
- `JWT_SECRET`
- `HCA_CLIENT_ID`
- `HCA_CLIENT_SECRET`

Optional variable:

- `BIND_ADDR` defaults to `0.0.0.0:8080`
- `DNS_BIND_ADDR` defaults to `0.0.0.0:5353`
- `AUTO_MIGRATE` controls whether startup applies pending migrations.

Notes:

- `HCA_REDIRECT_URI` must use `http` or `https`.
- The JWT secret is parsed as a base64 secret.
- `DATABASE_URL` is required by the application.

## Running Locally

- Application binary: `cargo run -p hackflare-app`
- Application build: `cargo build -p hackflare-app`
- Application tests: `cargo test -p hackflare-app`
- Docker dev application: `docker compose -f deployment/compose.dev.yml up -d`

## Notes

- The retired Slack webhook integration is no longer part of the application.
- The API is served by the same process as the SSR frontend.
