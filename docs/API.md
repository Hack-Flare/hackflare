# Hackflare Backend API

This document describes the backend that is currently implemented in [hackflare_api](../hackflare_api). If this file ever disagrees with the Rust source, trust the source.

## Current Surface Area

The backend entrypoint is [hackflare_api/src/main.rs](../hackflare_api/src/main.rs). HTTP routing is assembled in [hackflare_api/src/routes/mod.rs](../hackflare_api/src/routes/mod.rs), with auth in [hackflare_api/src/routes/auth.rs](../hackflare_api/src/routes/auth.rs), user routes in [hackflare_api/src/routes/users.rs](../hackflare_api/src/routes/users.rs), and JWT cookie validation in [hackflare_api/src/middlewares/auth.rs](../hackflare_api/src/middlewares/auth.rs).

At the moment, the backend exposes only these HTTP routes:

- `GET /api/v1/auth/login`
- `GET /api/v1/auth/callback`
- `GET /api/v1/users/me`

There are no health, DNS, or database-backed CRUD endpoints implemented in the current Rust source tree.

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
- The cookie is validated by [hackflare_api/src/middlewares/auth.rs](../hackflare_api/src/middlewares/auth.rs).

Response:

```json
{ "id": "<user-id>" }
```

Common failure codes:

- `401 missing_jwt`
- `401 invalid_jwt`

## Session And Cookie Details

- The OAuth session state is stored in an in-memory `tower_sessions::MemoryStore`.
- Session entries expire after 15 minutes of inactivity.
- The issued `jwt` cookie is `HttpOnly`, uses `SameSite=Lax`, and is marked `Secure` when `API_HCA_REDIRECT_URI` uses `https`.
- The JWT itself is valid for 24 hours.

## Configuration

The backend reads configuration from environment variables in [hackflare_api/src/config.rs](../hackflare_api/src/config.rs).

Required variables:

- `API_HCA_REDIRECT_URI`
- `API_JWT_SECRET`
- `API_HCA_CLIENT_ID`
- `API_HCA_CLIENT_SECRET`

Optional variable:

- `API_BIND_ADDR` defaults to `0.0.0.0:8080`

Notes:

- `API_HCA_REDIRECT_URI` must use `http` or `https`.
- The JWT secret is parsed as a base64 secret.
- The sample `.env` file also lists `DATABASE_URL` and `API_DNS_BIND_ADDR`, but the current Rust backend does not use them yet.

## Running Locally

- Backend binary: `cargo run -p hackflare-api`
- Backend build: `cargo build -p hackflare-api`
- Backend tests: `cargo test -p hackflare-api`
- Docker dev backend: `docker compose -f compose.dev.yml --profile backend up -d`

## What Is Not Implemented Yet

- Persistent user storage
- Database-backed auth data
- DNS management endpoints
- Health and ping endpoints
- Any `Authorization: Bearer ...` API surface for the current routes
