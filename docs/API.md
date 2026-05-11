# Hackflare Backend API

This document describes the HTTP API exposed by the backend service. All endpoints return JSON and standard HTTP status codes. Error responses use the shape:

```json
{ "error": "error_code" }
```

Common headers
- `x-internal-token`: required for internal gateway-to-backend requests (where noted).
- `Authorization: Bearer <token>`: required for endpoints that operate on behalf of a user (see `GET /api/v1/auth/me`).

Top-level endpoints

- `GET /health`
  - Purpose: Simple health check. Public.
  - Response: `{ status: "ok", service: "hackflare-backend", visibility: "internal-only" }`

- `GET /api/v1/ping`
  - Purpose: API liveliness + whether DB is configured.
  - Headers: `x-internal-token` (gateway)
  - Response: `{ status: "ok", service: "hackflare-backend", database_configured: true }`

Authentication

Types

- `RegisterInput`
  ```json
  { "email": "user@example.com", "password": "password123" }
  ```

- `LoginInput`
  ```json
  { "email": "user@example.com", "password": "password123" }
  ```

- `EmailLoginRequest`
  ```json
  { "email": "user@example.com" }
  ```

- `EmailLoginVerification`
  ```json
  { "email": "user@example.com", "code": "123456" }
  ```

- `Session` (response)
  ```json
  {
    "token": "uuid-token",
    "user": { "id": 1, "email": "user@example.com", "is_admin": false }
  }
  ```

- `EmailLoginChallenge` (response)
  ```json
  { "email": "user@example.com", "code": "123456", "expires_in_seconds": 900 }
  ```

Endpoints

- `POST /api/v1/auth/register`
  - Headers: `x-internal-token`
  - Body: `RegisterInput`
  - Success: `200 OK` with `Session` JSON.
  - Errors: `400`/`401` depending on validation (see code `AuthError` mapping).

- `POST /api/v1/auth/login`
  - Headers: `x-internal-token`
  - Body: `LoginInput`
  - Success: `200 OK` with `Session`.

- `POST /api/v1/auth/email/request`
  - Headers: `x-internal-token`
  - Body: `EmailLoginRequest`
  - Success: `200 OK` with `EmailLoginChallenge` (code returned for testing/dev; in production this would be emailed).

- `POST /api/v1/auth/email/verify`
  - Headers: `x-internal-token`
  - Body: `EmailLoginVerification`
  - Success: `200 OK` with `Session`.

- `GET /api/v1/auth/hackclub/url`
  - Headers: `x-internal-token`
  - Response: `{ "url": "https://hackclub/..." }` when Hack Club is configured.

- `GET /api/v1/auth/hackclub/callback?code=<code>`
  - Headers: `x-internal-token`
  - Query: `code` from Hack Club OAuth flow
  - Success: `200 OK` with `Session` on successful exchange and JWT verification.

- `GET /api/v1/auth/me`
  - Headers: `Authorization: Bearer <token>`
  - Success: `200 OK` with the `User` object: `{ id, email, is_admin }`.

DNS Management

Types

- `Zone` (server representation)
  ```json
  {
    "id": 1,
    "name": "example.com",
    "user_id": 1,
    "ns_verified": false,
    "records": [ /* DnsRecord[] */ ]
  }
  ```

- `DnsRecord`
  ```json
  { "name": "www.example.com", "record_type": "A", "value": "1.2.3.4", "ttl": 60 }
  ```

- `NewRecordInput`
  ```json
  { "name": "www", "record_type": "A", "value": "1.2.3.4", "ttl": 60 }
  ```

- `ResolvedRecord` (DNS query response in API)
  ```json
  { "zone": "example.com", "name": "www.example.com", "record_type": "A", "value": "1.2.3.4", "ttl": 60 }
  ```

Endpoints

- `GET /api/v1/dns/zones`
  - Headers: `x-internal-token` and `Authorization: Bearer <token>`
  - Response: `200 OK` with `Zone[]` belonging to the authenticated user.

- `POST /api/v1/dns/zones`
  - Headers: `x-internal-token` and `Authorization: Bearer <token>`
  - Body: `{ "name": "example.com" }`
  - Success: `200 OK` with created `Zone`.

- `POST /api/v1/dns/zones/{zone_name}/records`
  - Headers: `x-internal-token` and `Authorization: Bearer <token>`
  - Body: `NewRecordInput`
  - Success: `200 OK` with updated `Zone`.
  - Notes: Zone must be verified before adding records.

- `POST /api/v1/dns/zones/{zone_name}/verify`
  - Headers: `x-internal-token` and `Authorization: Bearer <token>`
  - Purpose: Mark NS delegation verified. Success: updated `Zone`.

- `GET /api/v1/dns/records?name=<fqdn>&record_type=<TYPE>`
  - Headers: `x-internal-token`
  - Query params: `name` required; `record_type` optional (A, AAAA, CNAME, TXT, NS, PTR, MX)
  - Response: `200 OK` with `ResolvedRecord[]`.

Nameserver

- The authoritative DNS UDP nameserver runs in the same process and answers queries from the in-memory (and persisted) zones/records. Supported query types: `A`, `AAAA`, `CNAME`, `TXT`, `ANY`.

Config & Environment

- `DATABASE_URL` (required): PostgreSQL connection string used for durable backend state.
- `BACKEND_BIND_HOST` (default `0.0.0.0`)
- `BACKEND_BIND_PORT` (default `8080`)
- `BACKEND_DNS_BIND_HOST` (default `0.0.0.0`)
- `BACKEND_DNS_BIND_PORT` (default `5353`)
- Email and HackClub-related vars: see `Config` in source.

Errors

- Errors map to HTTP status codes via `map_auth_error` and `map_dns_error` in `src/app.rs`. Common codes include `400`, `401`, `404`, `409`, `422`, and `502` for upstream failures.

Examples

- Create a zone (curl example):

```bash
curl -X POST "http://localhost:8080/api/v1/dns/zones" \
  -H "x-internal-token: <gateway-token>" \
  -H "Authorization: Bearer <user-token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"example.com"}'
```

Notes
- The backend requires `DATABASE_URL` at startup; state is persisted to PostgreSQL.
- Authentication tokens are UUID strings issued by the backend; these are stored server-side in the sessions map and persisted.
