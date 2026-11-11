# Hackflare API

Relevant source:

- Entrypoint: [hackflare-app/src/main.rs](../hackflare-app/src/main.rs)
- API routing: [hackflare-app/src/api/routes/mod.rs](../hackflare-app/src/api/routes/mod.rs)
- Auth routing: [hackflare-app/src/auth/routes.rs](../hackflare-app/src/auth/routes.rs)
- Auth middleware: [hackflare-app/src/auth/middleware.rs](../hackflare-app/src/auth/middleware.rs)
- Configuration: [hackflare-app/src/config.rs](../hackflare-app/src/config.rs)

## Route families

| Prefix | Purpose |
| --- | --- |
| `/api/v1/health` | Liveness and database check |
| `/api/v1/auth` | OAuth, email login, registration, password reset, session cookies |
| `/api/v1/users` | Current user profile |
| `/api/v1/sessions` | Login sessions for the current user |
| `/api/v1/dns` | DNS zones and records |
| `/api/v1/settings` | API keys and password |
| `/api/v1/logs` | DNS query logs |
| `/api/v1/notifications` | User notifications |
| `/api/v1/traffic` | Traffic analytics for the current user |
| `/api/v1/admin` | Admin users, statistics, and platform-wide traffic |

## Authentication

Two mechanisms are accepted.
The auth middleware checks API keys first, then falls back to the `jwt` cookie.

### Session cookies

The browser logs in through Hack Club Auth (HCA) or email and password.
On success the API sets two cookies:

- `jwt` - access token, path `/`, default lifetime 15 minutes.
- `refresh_jwt` - refresh token, path `/api/v1/auth`, default lifetime 30 days.

Both cookies are `HttpOnly` and `SameSite=Lax`, and are `Secure` when `HCA_REDIRECT_URI` uses `https`.
`POST /api/v1/auth/refresh` issues a fresh pair for the same session.
`POST /api/v1/auth/logout` revokes the session and clears both cookies.

### API keys

Scripts and integrations use a key from `POST /api/v1/settings/api-keys`:

```
Authorization: Bearer <raw_key>
```

An API key authenticates as its owning user.
The middleware records `last_used_at` and attaches a virtual session that expires one year from the request.

## Error format

- Auth handlers and the auth middleware return a bare status code with a plain-text body, for example `401 missing_jwt`.
- JSON handlers return `{"error": "<code>"}`.
- Some routes return a bare status with no body, such as `204 No Content`.

## Endpoints

### Health

`GET /api/v1/health` - public.

```json
{ "status": "ok", "database": "ok", "dns_zones": 0 }
```

`status` is `ok` when the database check passes and `degraded` otherwise.
`database` is `ok` or `error: <message>`.

### Auth

`GET /api/v1/auth/login` - starts the HCA OAuth redirect.

- Query: `target` optional.
  Stored in the session and used as the post-login redirect target.
- Generates a CSRF token, stores it in an in-memory session, and redirects to `https://auth.hackclub.com/oauth/authorize`.
- Response: `302 Found`.

`POST /api/v1/auth/login` - email and password login.

- Body: `{ "email": "...", "password": "..." }`.
- Response: `200 OK` with the `jwt` and `refresh_jwt` cookies.
- Errors: `400 email_and_password_required`, `401 invalid_email_or_password`.

`POST /api/v1/auth/register` - creates an email and password account.

- Body: `{ "email": "...", "password": "...", "first_name": "...", "last_name": "..." }`.
- Password must be at least 8 characters.
- Response: `201 Created` with the `jwt` and `refresh_jwt` cookies.
- Errors: `400 invalid_email`, `400 password_too_short`, `400 name_required`, `409 email_already_registered`.

`GET /api/v1/auth/callback` - finishes the OAuth exchange.

- Query: `code` and `state` required.
  `state` must match the CSRF token from `GET /api/v1/auth/login`.
- Exchanges the code for an HCA access token, fetches the profile, upserts the user, creates a session, and sets the `jwt` and `refresh_jwt` cookies.
- Response: `302 Found` to the stored target, or `/` when no target was set.
- Errors: `400 missing_auth_state`, `400 csrf_token_mismatch`, `400 exchange_failed`, `400 hca_rejected_exchange`, `401 hca_identity_denied`, `500 identity_request_failed`, `500 invalid_user_data`, `500 token_parse_failed`, `500 jwt_encode_error`.

`POST /api/v1/auth/refresh` - rotates the session cookies.

- Requires the `refresh_jwt` cookie.
- Response: `200 OK` with a fresh `jwt` and `refresh_jwt`.
- Errors: `401 missing_refresh_token`, `401 invalid_refresh_token`, `401 invalid_token_type`, `401 session_invalid`.

`POST /api/v1/auth/logout` - revokes the session and clears both cookies.

- Response: `204 No Content`.

`POST /api/v1/auth/forgot-password` - requests a password reset email.

- Body: `{ "email": "..." }`.
- Always responds `200 OK` with `{"message": "if the email exists, a reset link has been sent"}`.
  The response does not reveal whether the email exists.
- When SMTP is not configured, no email is sent.

`POST /api/v1/auth/reset-password` - sets a new password from a reset token.

- Body: `{ "token": "...", "password": "..." }`.
- Password must be at least 8 characters.
- All sessions for the user are revoked.
- Response: `200 OK` with `{"message": "password reset successfully"}`.
- Errors: `400 token_required`, `400 password_too_short`, `400 invalid_or_expired_token`.

### Users

`GET /api/v1/users/me` - requires authentication.

```json
{
  "id": "<user-id>",
  "first_name": "Ada",
  "last_name": "Lovelace",
  "email": "ada@example.com",
  "eligible": false,
  "has_password": true,
  "is_admin": false,
  "created_at": "2026-01-01T00:00:00Z"
}
```

`eligible` reflects the Hack Club YSWS eligibility flag.
`is_admin` is true when the user has the admin flag or their email is listed in `ADMIN_EMAILS`.

### Sessions

`GET /api/v1/sessions` - requires authentication.

- Query: `limit` default 20, `offset` default 0.
- Response: a JSON array of sessions owned by the current user.

`GET /api/v1/sessions/{id}` - requires authentication.

- Response: the session, or `404 session_not_found`.
- Returns `403 forbidden` when the session belongs to another user.

A session object has `id`, `user_id`, `ip_address`, `expires_at`, `created_at`, and `revoked_at`.

### DNS

Every DNS route requires authentication and acts only on zones owned by the current user.
Record writes require the zone to be verified.

`GET /api/v1/dns/zones` - lists the current user's zones.

```json
[{ "name": "example.com", "ns_verified": false }]
```

`POST /api/v1/dns/zones` - adds a zone.

- Body: `{ "name": "example.com" }`.
- Response: `201 Created` with `{"name": "example.com"}`.
- Errors: `400` (name required or invalid), `409 zone already exists`.

`DELETE /api/v1/dns/zones/{zone_name}` - deletes an owned zone.

- Response: `204 No Content`, or `404` when the zone is not owned.

`POST /api/v1/dns/zones/{zone_name}/verify` - checks NS delegation.

- Resolves the zone's authoritative nameservers and compares them against `DNS_NAMESERVERS`.
- On a match, marks the zone verified, notifies the owner, and returns `{"verified": true, "message": "..."}`.
- On a mismatch, returns `{"verified": false, "message": "..."}`.
- Errors: `404` when the zone is not owned.

`GET /api/v1/dns/zones/{zone_name}/records` - lists records.

```json
[{ "id": "<uuid>", "name": "www", "type": "A", "value": "198.51.100.1", "ttl": 3600, "status": "active" }]
```

`POST /api/v1/dns/zones/{zone_name}/records` - creates a record.

- Body: `{ "name": "www", "type": "A", "value": "198.51.100.1", "ttl": 3600 }`.
- Response: `201 Created` with the created record.
- Errors: `403` when the zone is not verified, `404` when the zone is not owned.

`PUT /api/v1/dns/zones/{zone_name}/records/{record_id}` - updates a record.

- Body: same shape as create.
- Response: `200 OK` with the updated record.
- Errors: `403 zone not verified`, `404` when the record or zone is missing.

`DELETE /api/v1/dns/zones/{zone_name}/records/{record_id}` - deletes a record.

- Response: `204 No Content`.
- Errors: `403 zone not verified`, `404` when the record or zone is missing.

### Settings

All settings routes require authentication.

`GET /api/v1/settings/api-keys` - lists the current user's API keys.

```json
[{ "id": "<uuid>", "name": "CI", "prefix": "hf_...", "created_at": "...", "last_used_at": null, "revoked": false }]
```

`POST /api/v1/settings/api-keys` - creates an API key.

- Body: `{ "name": "CI" }`.
- Response: `201 Created` with `{"key": { ... }, "raw_key": "<secret>"}`.
- The `raw_key` is returned once and is never shown again.

`DELETE /api/v1/settings/api-keys/{id}` - revokes a key.

- Response: `204 No Content`, or `404` when the key is not owned.

`PUT /api/v1/settings/password` - sets or changes the password.

- Body: `{ "current_password": "...", "new_password": "..." }`.
- `current_password` is required only when the account already has a password.
- Response: `200 OK` with `{"message": "password_updated"}`.
- Errors: `400 password_too_short`, `400 current_password_incorrect`.

### Logs

`GET /api/v1/logs/query-logs` - requires authentication.

- Query: `zone` optional.
  When set, only that zone's queries are returned.
- Returns the last 24 hours, up to 200 entries, for zones owned by the user.

```json
{
  "logs": [
    { "id": 1, "timestamp": "2026-01-01 00:00:00 UTC", "level": "info", "path": "www.example.com", "zone": "example.com", "status": 0, "ms": 2 }
  ],
  "summary": { "errors_today": 0, "warnings_today": 0, "info_today": 1 }
}
```

`level` is `info` for `NOERROR`, `warning` for `NXDOMAIN`, and `error` otherwise.
`status` is the numeric DNS response code.

### Notifications

All notification routes require authentication and act on the current user's notifications.

- `GET /api/v1/notifications` - up to 50 recent notifications.
- `GET /api/v1/notifications/unread-count` - `{"count": 0}`.
- `PUT /api/v1/notifications/{id}/read` - marks one notification read.
- `PUT /api/v1/notifications/read-all` - marks all notifications read.

A notification has `id`, `user_id`, `title`, `message`, `notif_type`, `read`, `link`, and `created_at`.

### Traffic

All user traffic routes require authentication and cover the current user's zones.

- `GET /api/v1/traffic/summary` - `{ "total_requests": 0, "avg_processing_ms": 0.0, "success_rate": 0.0, "error_rate": 0.0 }`.
- `GET /api/v1/traffic/timeseries` - query `days` default 30, clamped to 1-365.
  Returns `[{ "date": "2026-01-01", "requests": 0, "errors": 0, "nxdomain": 0 }]`.
- `GET /api/v1/traffic/by-zone` - `[{ "zone": "example.com", "requests": 0, "errors": 0, "avg_ms": 0.0 }]`.
- `GET /api/v1/traffic/top-queries` - query `limit` default 10, clamped to 1-100.
  Returns `[{ "query": "www.example.com", "count": 0 }]`.

### Admin

All admin routes require authentication and the admin role.
A user is an admin when their account has the admin flag or their email is listed in `ADMIN_EMAILS`.
Non-admins receive `403 forbidden`.

- `GET /api/v1/admin/users` - all users as `{ "id", "email", "first_name", "last_name", "status", "is_admin", "created_at" }`.
- `POST /api/v1/admin/users/admin` - body `{ "email": "...", "admin": true }`.
  `admin` defaults to `true` and grants or revokes the admin flag.
  Errors: `400 email_required`, `400 cannot_remove_self`, `404 user_not_found`.
- `GET /api/v1/admin/stats` - `{ "total_users": 0, "total_zones": 0, "total_sessions": 0 }`.
- `GET /api/v1/admin/traffic/summary` - platform-wide traffic summary.
- `GET /api/v1/admin/traffic/timeseries` - query `days` default 30, clamped to 1-365.
- `GET /api/v1/admin/traffic/top-queries` - query `limit` default 10, clamped to 1-100.

## Configuration

The application reads configuration from environment variables in [hackflare-app/src/config.rs](../hackflare-app/src/config.rs).
See [.env.example](../.env.example) for a starting point.

Required:

| Variable | Notes |
| --- | --- |
| `DATABASE_URL` | Must use the `postgres` scheme. |
| `JWT_SECRET` | Base64 secret. Generate with `openssl rand -base64 32`. |
| `HCA_CLIENT_ID` | Hack Club Auth client id. |
| `HCA_CLIENT_SECRET` | Hack Club Auth client secret. |
| `HCA_REDIRECT_URI` | Must use `http` or `https`. The callback URL registered with HCA. |

Optional:

| Variable | Default | Notes |
| --- | --- | --- |
| `ENVIRONMENT` | `production` | `development` or `production`. |
| `AUTO_MIGRATE` | true in production | Applies pending migrations at startup. |
| `BIND_ADDR` | `0.0.0.0:8080` | HTTP bind address. |
| `DNS_BIND_ADDR` | `0.0.0.0:5353` | DNS bind address. |
| `FRONTEND_STATIC_DIR` | `<crate>/static` | Static asset directory. |
| `CLIENT_IP_SOURCE` | `ConnectInfo` | How the client IP is resolved. |
| `DISABLE_HOME_PAGE` | `false` | When true, `/` redirects to `/dash`. |
| `SESSION_INACTIVITY_MINUTES` | `15` | OAuth session inactivity window. |
| `ACCESS_TOKEN_MINUTES` | `15` | `jwt` cookie lifetime. |
| `REFRESH_TOKEN_DAYS` | `30` | `refresh_jwt` cookie lifetime. |
| `DNS_NAMESERVERS` | `ns1.hackflare.net,ns2.hackflare.net` | Expected nameservers for zone verification. |
| `ADMIN_EMAILS` | empty | Comma-separated emails treated as admins. |
| `DOMAIN` | unset | Public origin for password reset links. Bare hostnames default to `https`. |
| `CDN_URL` | unset | CDN origin for `/static/` assets. |
| `SMTP_HOST` | unset | When set, enables password reset email. |

When `SMTP_HOST` is set, `SMTP_PORT` (default `587`), `SMTP_USERNAME`, `SMTP_PASSWORD`, and `SMTP_FROM` are also required.

`DOMAIN` and `CDN_URL` accept a full `http` or `https` URL or a bare hostname.
Bare hostnames default to HTTPS and explicit ports are preserved.
Neither may include a path, query, or fragment.

When `CDN_URL` is set, rendered HTML rewrites `/static/` asset URLs to the CDN origin.
The CDN origin is added to the allowed CORS origins together with `DOMAIN`.
The CDN must serve `/static/*` with `Access-Control-Allow-Origin` for the application origin.

The DNS library reads its own `DNS_*` settings for SOA fields, recursion, and timeouts.
See [hackflare-dns](../hackflare-dns).

## Running locally

Run from the repository root with the [justfile](../justfile) recipes:

- `just run-app` - run the merged application using the repository `.env`.
- `just run-dev` - run with an embedded PostgreSQL database in development mode.
- `just db-up` / `just db-down` - start or stop the development PostgreSQL.
- `just build` - build the merged application.
- `just test-app` - test the merged application.
