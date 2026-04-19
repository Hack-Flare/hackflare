# HackFlare

Cloudflare alternative for HackClub. Hence the name HackFlare.  

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

This runs elixir and Rust code together  
```
mix phx.server
```

This runs just rust code  
FYI this does not work yet  
```
cd native/core && cargo run
```

## Needed Features for POC/MVP

- [ ] DNS System
  - [x] Backend
  - [ ] Frontend
- [ ] nameservers
  - [x] Working nameserver implementation
- [ ] Auth system (hc auth)
  - [ ] Backend
  - [ ] Frontend
- [ ] Simple Logging
- [ ] Caching (incl. DNS caching, Edge caching, etc.)
- [ ] Simple Website

## Features From Reccomendations

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