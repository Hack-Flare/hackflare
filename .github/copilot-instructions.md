# Copilot instructions for Hackflare

## Build, test, and lint commands

Use Elixir/Phoenix commands from repo root unless noted.

| Task | Command |
| --- | --- |
| Initial setup | `mix setup` |
| Run app (dev) | `iex -S mix phx.server` |
| Build/compile (CI parity) | `mix compile --warnings-as-errors` |
| Build static assets | `mix assets.build` |
| Production asset build | `mix assets.deploy` |
| Full test suite | `mix test` |
| Single test file | `mix test test/hackflare_web/controllers/page_controller_test.exs` |
| Single test at line | `mix test test/hackflare_web/controllers/page_controller_test.exs:10` |
| Elixir formatter | `mix format` |
| Elixir lint (strict) | `mix credo --strict` |
| Dependency audit | `mix deps.audit` |
| Security scan | `mix sobelow --config` |
| Type checks | `mix dialyzer` |
| Rust compile checks | `cargo check` |
| Rust lint (deny warnings) | `cargo clippy -- -D warnings` |

CI-specific test partitioning uses:

```bash
MIX_TEST_PARTITION=1 mix test --partitions 4
```

Database setup used in CI before tests:

```bash
mix do app.start + ecto.create + ecto.migrate
```

## High-level architecture

Hackflare is a Phoenix app with a Rust DNS engine loaded through Rustler:

1. `Hackflare.Application` supervision tree starts `Hackflare.Repo`, telemetry, `DNSCluster`, `Hackflare.Nameserver`, PubSub, and `HackflareWeb.Endpoint`.
2. `Hackflare.Nameserver` creates a Rust DNS manager via `Hackflare.Native.manager_new/0` and starts DNS serving with `manager_start_nameserver/3`.
3. Rust code in `native/core` (`dns/*`, `ns/*`, `nifs.rs`) handles DNS query parsing, lookup, recursion, and UDP/TCP server loops.
4. Web requests are served by Phoenix (`HackflareWeb.Router`, controllers, templates/components). Auth flow is OIDC via Assent (`Hackflare.HackClubAuth` + `AuthController`).
5. Persistence is PostgreSQL via Ecto (`Hackflare.Repo`, `Hackflare.Accounts.User`, migrations under `priv/repo/migrations`).

Docs are served from `/docs`:
- In dev, from `doc/` (endpoint dev-only static plug).
- In releases/Docker, copied into `priv/static/docs`.

## Key conventions in this repository

- **Rust NIF contract is runtime truth:** `Hackflare.Native` stubs exist in Elixir, but real behavior is defined in `native/core/src/nifs.rs`. In particular, several NIFs return booleans, and `manager_list_zones/1` + `manager_find_records/3` return JSON strings (decode before structured use).
- **Nameserver starts automatically with the app:** DNS server startup is not a manual task; it is started in `Hackflare.Nameserver.init/1` from supervision.
- **SOA config path is Elixir -> env vars -> Rust engine:** `config :hackflare, :dns` is exported to `HACKFLARE_DNS_SOA_*` env vars in `Hackflare.Nameserver`, then read in Rust `dns/engine.rs`.
- **Auth/session convention:** authenticated state is session `:user_id`; protected browser routes use `HackflareWeb.Plugs.RequireAuthenticated`.
- **Route split matters:** public browser routes and authenticated browser routes are separate scopes in `router.ex`; keep new protected pages in the authenticated scope.
- **Formatting/doc culture is enforced in automation:** formatter plugin includes `Phoenix.LiveView.HTMLFormatter`; CI also has an AI docs workflow that injects missing `@doc` entries for changed `lib/**/*.ex` files.
