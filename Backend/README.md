# Hackflare Backend

Internal backend service for Hackflare.

## Stack

- Rust 2024
- Axum

## Environment variables

- `BACKEND_BIND_HOST` default `0.0.0.0`
- `BACKEND_BIND_PORT` default `8080`
- `BACKEND_GATEWAY_TOKEN` required for internal routes
- `DATABASE_URL` optional for now

## Routes

- `GET /health`
- `GET /internal/v1/ping` requires `x-internal-token` header

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
