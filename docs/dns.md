# Hackflare DNS (hackflare_dns)

Overview
--------

`hackflare_dns` is an authoritative + recursive DNS crate built on top of `hickory-server`. It provides an embeddable `Nameserver` API for managing zones and serving DNS queries, with optional persistence backends.

Key concepts
- `Nameserver` — high-level API (create zones, add/remove records, run server)
- `DnsConfig` — engine configuration (recursion, SOA defaults, timeouts, EDNS size)
- `NsConfig` — nameserver config (bind address, port, zone file, database URL)
- Persistence — pluggable `ZonePersistence` trait with `PostgresPersistence` and `MemoryPersistence`

Important modules
- `hackflare_dns::dns::engine` — DNS packet parsing/response logic and recursive resolver glue
- `hackflare_dns::dns::manager` — in-memory manager for zones and records
- `hackflare_dns::dns::records` — encoders for record types (A, AAAA, MX, SOA, TLSA, etc.)
- `hackflare_dns::ns::server` — `Nameserver` wrapper that integrates engine + authority store + runtime
- `hackflare_dns::ns::persistence` — persistence trait and implementations (Postgres, Memory)

Nameserver API (high level)
- `Nameserver::new(config: NsConfig)` — create nameserver with default `DnsConfig` (in-memory zones)
- `Nameserver::with_dns_config(config, dns_config)` — use custom DNS engine settings
- `Nameserver::with_persistence(config, dns_config, persistence)` — enable durable storage
- `create_zone(name)` — create an authoritative zone with default SOA
- `add_record(zone, name, type, ttl, data)` — add/modify a record
- `remove_record(zone, name, type)` — delete a record
- `list_zones()` — list hosted zones
- `load_zones_from_storage()` — restore zones from persistence (when configured)
- `run()` — start the DNS server (blocks)

Configuration via environment
See `DnsConfig::from_env()` for all supported variables. Highlights:
- `HACKFLARE_DNS_RECURSION_ENABLED` — enable recursive resolution
- `HACKFLARE_DNS_UDP_SIZE` — EDNS UDP size
- `HACKFLARE_DNS_RECURSION_ROUNDS` — max recursion rounds
- `HACKFLARE_ROOT_HINTS_FILE` — optional path to root-hints
- `DATABASE_URL` — used by `PostgresPersistence` if provided

Persistence
- `ZonePersistence` trait defines async methods: load_zones, load_zone, save_zone, delete_zone, save_record, delete_record
- `PostgresPersistence` — production-ready PostgreSQL backend; `init_schema()` creates required tables
- `MemoryPersistence` — simple in-memory implementation for tests and development

Running & examples
- Build: `cargo build -p hackflare-dns`
- Tests: `cargo test -p hackflare-dns`
- Example usage is documented in `hackflare_dns/src/lib.rs` and `ns/server.rs` (see code examples for in-memory and Postgres-backed setups)

Notes
- The crate exposes low-level helpers (record encoders, qname parsing) and includes tests for the DNS engine and persistence layers
- For production DNS serving, run the `Nameserver::run()` method from a small binary that composes `NsConfig` and optional persistence
