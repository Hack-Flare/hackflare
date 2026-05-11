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
- [x] nameservers
  - [ ] Speed improvments
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