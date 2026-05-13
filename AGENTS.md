# Hackflare Agent Instructions

Use this file as the default guide for AI agents working in the repository.

## Repository Shape

- Backend lives in [hackflare_api](hackflare_api); frontend lives in [frontend](frontend).
- Prefer the source tree over documentation when they disagree. In particular, [docs/API.md](docs/API.md) is outdated and should not be treated as the source of truth for backend behavior.
- For frontend-specific conventions, follow [frontend/CLAUDE.md](frontend/CLAUDE.md).

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

## Frontend Workflow

- Follow the existing React Router + shadcn/ui patterns in [frontend/CLAUDE.md](frontend/CLAUDE.md).
- Frontend scripts are defined in [frontend/package.json](frontend/package.json).

## Editing Guidance

- Keep changes minimal and localized.
- Link to existing docs instead of duplicating them.
- If backend docs need to be corrected, update the source-backed docs or code comments rather than adding a second conflicting description.
