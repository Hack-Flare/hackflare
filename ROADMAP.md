## Roadmap

Feel free to submit PRs/issues for anything you want us to work on or you want to work on.

> IMPORTANT  
> NOTE: THIS IS ONLY EMPTY DUE TO THE REFACTOR WE HAVE BEEN DOING

### Phase 1 (MVP/POC)

- [ ] DNS System
  - [x] Backend
    - [x] Auth Integration
    - [x] Domain Management
      - Domains stay in a pending state until their NS delegation is verified, and record edits are blocked until then.
  - [ ] Needs Testing
- [ ] **DNS Crate** (`hackflare_dns/`)
  - [x] Authoritative zone management (hickory-backed `AuthorityStore`)
  - [x] Record type encoding (A, AAAA, CNAME, MX, TXT, SOA, SRV, etc.)
  - [x] Recursive resolver (legacy engine - `dns/recursive.rs`)
  - [x] PostgreSQL persistence (`PostgresPersistence`)
  - [x] In-memory persistence (`MemoryPersistence`)
  - [x] Hickory server integration (`ns/hickory.rs`)
  - [ ] **Testing & Reliability**
    - [ ] Unit tests for `dns/engine.rs` (DNS wire protocol encoding/decoding)
    - [ ] Integration tests for recursive resolver (mock upstream servers)
    - [ ] End-to-end tests with hickory-server transport
    - [ ] Fuzz testing for DNS message parsing
  - [ ] **Security Hardening**
    - [ ] Filter bogon/private IPs in recursive resolver (SSRF prevention)
    - [ ] Parameterized SQL in metrics flusher (`hickory.rs:63-69`)
    - [ ] TLS support for PostgreSQL connections (replace `NoTls`)
    - [ ] Input validation: TTL range checks, domain name format validation
    - [ ] Defensive bounds checks in record encoding (buffer overflows)
  - [ ] **Persistence Improvements**
    - [ ] Full record export/import in `save_zone_to_storage` (currently saves zone name only)
    - [ ] Connection pooling for PostgreSQL (instead of connect-per-call)
    - [ ] Zone file import/export (Bind-compatible zone files)
  - [ ] **Performance**
    - [ ] DNS response caching (recursive resolver)
    - [ ] Cache hit/miss metrics
    - [ ] Replace `once_cell::sync::Lazy` with `std::sync::LazyLock`
    - [ ] Reduce redundant allocations in record encoding hot path
    - [ ] Async PostgreSQL driver (`tokio-postgres` instead of blocking `postgres`)
  - [ ] **Recursive Resolver**
    - [ ] EDNS0 support (payload size negotiation)
    - [ ] DNSSEC validation (authenticated data AD bit)
    - [ ] Configurable upstream DNS forwarders
    - [ ] TCP fallback for truncated UDP responses
    - [ ] Rate limiting and query throttling
  - [ ] **Observability**
    - [ ] Structured tracing integration (tracing crate instead of JSON `eprintln!`)
    - [ ] Per-zone query statistics
    - [ ] Recursion latency histogram
    - [ ] Prometheus metrics endpoint
  - [ ] **New Features**
    - [ ] DNS-over-TLS (DoT) support
    - [ ] DNS-over-HTTPS (DoH) support
    - [ ] Zone transfer (AXFR/IXFR) support
    - [ ] NS delegation verification (pending → active zone state machine)
    - [ ] CNAME flattening/redirection API
    - [ ] Geo-based DNS routing (latency-based, region-based)
  - [ ] **Code Quality**
    - [ ] Fix clippy warnings (155 pedantic/nursery warnings)
    - [ ] Document internal modules (`dns/engine.rs`, `dns/recursive.rs`)
    - [ ] Standardize error types (replace `Box<dyn Error>` with custom error enum)
- [ ] Auth system
  - [x] Backend
    - [x] HackClub Auth
    - [x] Session Management
    - [ ] Github Auth
    - [x] Email Auth
    - [ ] Password Reset
    - [x] Email Verification
    - [ ] Google Auth
  - [ ] Frontend
    - [ ] Login/Signup Page
    - [ ] Dashboard Auth Integration
  - [ ] ENV Setup
- [ ] Simple Logging
- [ ] Proper Frontend
  - [ ] Dashboard
  - [ ] Domain Management
  - [ ] Logging
  - [ ] Notifications
  - [ ] Admin Panel
  - [ ] Settings
  - [ ] Error Pages
  - [ ] Auth System
- [x] Docker
- [ ] Big Haj on error pages
- [ ] Organize readme and documentation better

- [ ] Working Production

### Phase 2 (Post MVP)
- [ ] API
- [ ] Caching (incl. DNS caching, minimal site caching)
- [ ] DDoS Protection
- [ ] Load Balancing
- [ ] Clerk Integration - Maybe
- [ ] Tunneling
- [ ] Node Based Nameservers (All can connect to main server through api)
- [ ] Community Server Support
- [ ] Dynamic Firewall (Optional)
- [ ] Custom CDN
- [ ] Email Notifications
- [ ] Analytics
- [ ] Performance Monitoring
- [ ] SSL/TLS Management
- [ ] API Support (gRPC and REST)
- [ ] Team Management
- [ ] Live Logging

### Phase 3 
- [ ] Proxying
- [ ] IPv6 Support
- [ ] Serverless Functions
- [ ] Workers
- [ ] Turnstile Support
- [ ] Suspicious Traffic Detection and Blocking
- [ ] Custom DNS Records (SRV, TXT, etc.)

### Phase 4
- [ ] Email Routing and Sending
- [ ] Slack Bot
- [ ] Live Packet Watching (for fun)
- [ ] Pages
- [ ] SSL certificates

### Extra/Not sure when
- [ ] TMP Docker, a temporary docker for users to test stuff.
- [ ] ISO 27001:2022 certification?
- [ ] Anti Scanning/Scraping measures
- [ ] Custom Error Pages


## Stardance Phase
All stuff here should be done in stardance

- [ ] Captcha // Redac1ed
  - [ ] Core working
  - [ ] IP scanning (VPNs, proxies etc.)
  - [ ] JS/React SDK

- [ ] Registrar // SeradedStripes
  - [ ] Domain purchasing
  - [ ] Domain management (renewals, transfers, etc.)
  - [ ] Registrar API integration
  - [ ] Good Frontend for domain management