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

## Needed Features for POC/MVP

- [ ] DNS System
  - [ ] Backend
    - [ ] Auth Integration
    - [ ] Domain Migration
    - [ ] Domain Management
  - [ ] Frontend
- [x] nameservers
  - [1] Amount Online
- [ ] Auth system (hc auth)
  - [ ] Backend
  - [ ] Frontend
  - [x] ENV Setup
- [ ] Simple Logging
- [ ] Caching (incl. DNS caching, Edge caching, etc.)
- [ ] Proper Frontend
- [x] Docker

## Features From Recommendations

- Node Based Nameservers (All can connect to main server through api)
- Dynamic Firewall (Optional)
- DDoS Protection
- Load Balancing
- Proxy w/ multiple ips
- Possible IPv6, - maybe limit to HCB donors
- Tunneling
- Cloudflare migration tool
- Sharing Support (for domains and teams)
- turnstile support
- Auto block suspicious traffic and ip ranges 
- Live Logging
- Performance monitoring
- Email Routing and Sending
- Email Notifications
- Passkey Auth Second Layer
- Email Auth
- HC AI Integration
- HC CDN Integration
- gRPC and Rest API support
- Custom DNS Records (SRV, TXT, etc.)
- Serverless Functions (eg. DB, Workers, etc.)
- Pages
- Analytics
- SSL/TLS Management
- SSL certificates?
- Slack Bot
- Live Packet Watching (for fun)
- TMP Docker, a temporary docker for users to test stuff.
- ISO 27001:2022 certification?
- Anti Scanning/Scraping measures
- Custom Error Pages
