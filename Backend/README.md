# Hackflare Backend

Internal backend service for Hackflare.

## Stack

- Rust 2024
- Axum

## Environment variables

- `BACKEND_BIND_HOST` default `0.0.0.0`
- `BACKEND_BIND_PORT` default `8080`
- `BACKEND_DNS_BIND_HOST` default `0.0.0.0`
- `BACKEND_DNS_BIND_PORT` default `5353`
- `BACKEND_GATEWAY_TOKEN` required for gateway-to-backend requests
- `DATABASE_URL` optional for now
- `BACKEND_EMAIL_FROM_NAME` default `Hackflare`
- `BACKEND_EMAIL_FROM_ADDRESS` default `no-reply@hackflare.local`
- `BACKEND_EMAIL_REPLY_TO` optional
- `BACKEND_EMAIL_SMTP_HOST` optional
- `BACKEND_EMAIL_SMTP_PORT` default `587`
- `BACKEND_EMAIL_SMTP_USERNAME` optional
- `BACKEND_EMAIL_SMTP_PASSWORD` optional
- `BACKEND_EMAIL_SMTP_STARTTLS` default `true`

## DNS nameserver

- UDP authoritative DNS server runs in the same backend process.
- DNS answers come from `/api/v1/dns/*` managed zones/records.
- Supported query types: `A`, `AAAA`, `CNAME`, `TXT`, `ANY`.
- Unknown zones return `NXDOMAIN`.

## Routes

- `GET /health`
- `GET /api/v1/ping` requires `x-internal-token`
- `POST /api/v1/auth/register` requires `x-internal-token`
- `POST /api/v1/auth/login` requires `x-internal-token`
- `GET /api/v1/auth/me` requires `x-internal-token` and `Authorization: Bearer <token>`
- `GET /api/v1/dns/zones` requires `x-internal-token`
- `POST /api/v1/dns/zones` requires `x-internal-token`
- `POST /api/v1/dns/zones/:zone_name/records` requires `x-internal-token`
- `GET /api/v1/dns/records` requires `x-internal-token`

## Local run

```bash
cargo run
```

## checks

```bash
cargo fmt --all --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
