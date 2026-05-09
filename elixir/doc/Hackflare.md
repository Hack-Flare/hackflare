# `Hackflare`

Hackflare is a Cloudflare alternative built for HackClub.

This module serves as the top-level application context and provides the foundation
for the domain logic and business operations of the HackFlare DNS management system.

## Architecture

The Hackflare application is built on the PETRL stack:
- **Phoenix** - Web framework for HTTP/WebSocket communication
- **Elixir** - Functional language for business logic and OTP applications
- **Tailwind** - Utility-first CSS framework for styling
- **Rust** - Performance-critical DNS handling via native extensions
- **LiveView** - Real-time interactive web components

## Key Components

### Contexts
The application uses a context-based architecture where business logic is contained
in separate modules. Each context manages its own data and state.

### Native Integration
DNS operations (nameserver, zone management, record handling) are implemented in Rust
using Rustler NIFs for maximum performance. The `Hackflare.Native` module provides
the bindings to these native functions.

### Database
PostgreSQL is used via Ecto for persistent storage of domain, DNS record, and user data.
See `Hackflare.Repo` for database access.

## Running the Application

    iex -S mix phx.server

This starts the Phoenix server with the IEx console, allowing interactive development.

---

*Consult [api-reference.md](api-reference.md) for complete listing*
