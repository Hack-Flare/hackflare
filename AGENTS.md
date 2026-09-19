# Hackflare Agent Instructions

Use this file as the default guide for AI agents working in the repository.

## Repository Shape

- Backend HTTP API lives in [hackflare_api](hackflare_api); DNS server library lives in [hackflare_dns](hackflare_dns); frontend lives in [frontend](frontend).
- Prefer the source tree over documentation when they disagree. [docs/API.md](docs/API.md) is useful reference material, but docs can drift; trust the Rust source as the source of truth for behavior when they conflict.
- For frontend-specific conventions, follow [frontend/CLAUDE.md](frontend/CLAUDE.md).
- For DNS-specific conventions, check [hackflare_dns/src/lib.rs](hackflare_dns/src/lib.rs) for the module structure and high-level design.

## Backend Workflow

- Build with `cargo build -p hackflare-api`.
- Run with `cargo run -p hackflare-api`.
- Start the backend dev container with `docker compose -f compose.dev.yml --profile backend up -d`.
- Use `cargo test -p hackflare-api` for Rust tests, but note that the backend currently has little or no in-tree test coverage.
- The main backend entrypoint is [hackflare_api/src/main.rs](hackflare_api/src/main.rs), and HTTP routing is assembled in [hackflare_api/src/routes/mod.rs](hackflare_api/src/routes/mod.rs).
- Backend config is loaded from environment in [hackflare_api/src/config.rs](hackflare_api/src/config.rs); startup requires the HCA and JWT variables documented in [.env.example](.env.example).

## Backend Conventions

- Keep route behavior, config loading, and middleware logic close to the owning module.
- Trust the Rust source for current API shapes, auth flow, and cookie/session behavior.
- The backend uses `dotenv`, `reqwest`, `axum`, `tower-sessions`, and JWT cookies; inspect the existing modules before introducing new abstractions.

## DNS Workflow

- Build with `cargo build -p hackflare-dns`.
- Run tests with `cargo test -p hackflare-dns`.
- The DNS crate provides a library, not a standalone binary; it is integrated into [hackflare_api](hackflare_api) for production deployment.
- The main crate entrypoint is [hackflare_dns/src/lib.rs](hackflare_dns/src/lib.rs); DNS protocol handling lives in [hackflare_dns/src/dns](hackflare_dns/src/dns) and Nameserver implementation in [hackflare_dns/src/ns](hackflare_dns/src/ns).
- The Nameserver uses `hickory-server` as the transport/runtime wrapper; authoritative zones/records live behind Hickory's Catalog + InMemoryZoneHandler; the legacy engine is used only for recursive fallback.
- PostgreSQL backend for zone persistence is optional and configured via [hackflare_dns/src/ns/persistence.rs](hackflare_dns/src/ns/persistence.rs).
- Metrics are flushed to PostgreSQL dns_query_metrics table; request counting happens in the Hickory handler.

## DNS Conventions

- Keep protocol logic separate from persistence concerns; use the `dns/` module for protocol operations and `ns/` for the Nameserver runtime.
- Trust the Rust source and the Hickory library documentation for DNS message handling and caching behavior.
- The crate depends on `hickory-server`, `tokio`, `postgres`, and `serde`; avoid adding blocking I/O in async contexts.

## Frontend Workflow

- Follow the existing React Router + shadcn/ui patterns in [frontend/CLAUDE.md](frontend/CLAUDE.md).
- Frontend scripts are defined in [frontend/package.json](frontend/package.json).

## Editing Guidance

- Keep changes minimal and localized.
- Link to existing docs instead of duplicating them.
- If docs need to be corrected, update the source-backed docs or code comments rather than adding a second conflicting description.
