## Roadmap

Feel free to submit PRs/issues for anything you want us to work on or you want to work on.


### Phase 1 (MVP/POC)

- [x] DNS System
  - [x] Backend
    - [x] Auth Integration
    - [x] Domain Management
      - Domains stay in a pending state until their NS delegation is verified, and record edits are blocked until then.
- [x] **DNS Crate** (`hackflare_dns/`)
  - [x] Authoritative zone management (hickory-backed `AuthorityStore`)
  - [x] Record type encoding (A, AAAA, CNAME, MX, TXT, SOA, SRV, etc.)
  - [x] Recursive resolver (legacy engine - `dns/recursive.rs`)
  - [x] PostgreSQL persistence (`PostgresPersistence`)
  - [x] In-memory persistence (`MemoryPersistence`)
  - [x] Hickory server integration (`ns/hickory.rs`)
- [x] Auth system
  - [x] Backend
    - [x] HackClub Auth
    - [x] Session Management
    - [x] Email Auth
    - [x] Password Reset
    - [x] Email Verification
  - [x] Frontend
    - [x] Login/Signup Page
    - [x] Dashboard Auth Integration
  - [x] ENV Setup
- [x] Simple Logging
- [x] Proper Frontend
  - [x] Dashboard
  - [x] Domain Management
  - [x] Logging
  - [ ] Notifications
  - [x] Admin Panel
  - [x] Settings
  - [x] Error Pages
  - [x] Auth System
- [x] Docker

### Phase 2 (Post MVP)
- [ ] API
- [ ] Caching (incl. DNS caching, minimal site caching)
- [ ] DDoS Protection
- [ ] Load Balancing
- [ ] Tunneling
- [ ] Node Based Nameservers (All can connect to main server through api)
- [ ] Community Server Support
- [ ] Dynamic Firewall (Optional)
- [ ] Custom CDN
- [ ] Email Notifications
- [x] Analytics
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

