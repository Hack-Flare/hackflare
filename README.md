<div align="center">
<img alt="upscalemedia-transformed (1)" src="https://github.com/user-attachments/assets/d99965b5-523d-4878-b985-93a841477db2" />
</div>

# HackFlare
Cloudflare alternative for HackClub. Hence the name HackFlare.  
[View Elixir Documentation](ELIXIR_DOCS.md)

## Repo Structure

- `docs/` - Documentation when we do it
- `/` - Main codebase for the project, including backend and frontend code

## How we using Rust and Elixir

Elixir will be the main language with Rust for performance critical code.  
Using Rustler to integrate Rust code into Elixir and use Phoenix for the web framework.  
Will also integrate inline rust assembly code for performance critical sections.

### Stack

PETRL  
- Phoenix
- Elixir
- Tailwind
- Rust
- liveview

## Running the project

Without Docker
```
iex -S mix phx.server
```

With Docker from prebuilt image
```
docker compose up
```

Dev Docker Which builds the image locally
```
docker compose -f docker-compose.dev.yml up
```
## Roadmap

Feel free to submit PRs/issues for anything you want us to work on or you want to work on.

### Phase 1 (MVP/POC)

- [x] DNS System
  - [x] Backend
    - [x] Auth Integration
    - [x] Domain Management
      - Domains stay in a pending state until their NS delegation is verified, and record edits are blocked until then.
- [x] nameservers
- [x] Auth system (hc auth)
  - [x] Backend
  - [x] ENV Setup
- [ ] Simple Logging
- [wip] Proper Frontend
  - [wip] Dashboard
  - [wip] Domain Management
  - [ ] Logging
  - [ ] Notifications
  - [wip] Admin Panel
  - [ ] Settings
  - [x] Error Pages
  - [x] Auth System
- [x] Docker
- [x] Big Haj on error pages
- [semi-complete] Organize readme and documentation better

- [ ] Working Production

### Phase 2 (Post MVP)
- [ ] Caching (incl. DNS caching, minimal site caching)
- [ ] DDoS Protection
- [ ] Load Balancing
- [ ] Clerk Integration
- [ ] Tunneling
- [ ] Node Based Nameservers (All can connect to main server through api)
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
- [ ] Captcha
  - [ ] Core working
  - [ ] IP scanning (VPNs, proxies etc.)
  - [ ] JS/React SDK
