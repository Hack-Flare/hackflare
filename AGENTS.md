# AGENTS — Repo-level guidance for AI coding agents

Purpose
- Short, focused instructions to help an AI agent be productive in this repository.

Quick facts
- Primary languages: Elixir (Phoenix), Rust.
- Main services: backend API (`hackflare_api`) and Phoenix web components under `lib/`.

Quick start (development)
- Preferred: use Docker for reproducible environment:
  - `docker compose up` — start using the prebuilt images.
  - `docker compose -f docker-compose.dev.yml up` — local development build.
- Backend (Rust): see `hackflare_api/Cargo.toml` — `cargo build` / `cargo run` / `cargo test` in `hackflare_api`.
- Elixir/Phoenix: run `mix` commands in the workspace or app directories (e.g., `mix test`, `mix phx.server`). See project README.

Important environment notes
- The backend requires `DATABASE_URL` for PostgreSQL. See `docs/API.md` for runtime config keys and behavior.

Where to look (high-value files)
- Repository README: README.md — overall project and Docker commands.
- API reference: docs/API.md — HTTP endpoints, config, and examples.
- Rust backend crate: hackflare_api/Cargo.toml and `hackflare_api/src`.
- Elixir web app: lib/hackflare_web/ (Phoenix components and templates).
- Contribution guidelines: CONTRIBUTING.md.

Agent behavior guidance
- Link to existing docs; do not duplicate large docs — follow "link, don't embed."
- Prefer Docker-based instructions for local runs unless user requests native builds.
- When suggesting commands, include the working directory (e.g., `cd hackflare_api && cargo test`).
- Note required env vars (at least `DATABASE_URL`) and point to `docs/API.md`.

Suggested next customizations
- Add per-language agent instructions (Elixir vs Rust) if agents will run tests or format code.
- Add a small CI helper skill to run `docker compose -f docker-compose.dev.yml up --build` and smoke-test endpoints.

If anything important is missing or unclear in the docs, ask the user which area to focus on next.
