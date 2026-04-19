# Hackflare Elixir Documentation

## Overview

Hackflare is a Cloudflare alternative built for HackClub, implementing a DNS management system with a modern web interface. The Elixir codebase is built on the PETRL stack (Phoenix, Elixir, Tailwind, Rust, LiveView).

## Project Structure

```
hackflare/
├── lib/
│   ├── hackflare/              # Application context and business logic
│   │   ├── application.ex      # OTP application supervision tree
│   │   ├── repo.ex             # Ecto PostgreSQL repository
│   │   ├── mailer.ex           # Swoosh email service
│   │   ├── nameserver.ex       # DNS nameserver GenServer
│   │   └── native.ex           # Rust NIF bindings
│   ├── hackflare_web/          # Web layer (controllers, views, routes)
│   │   ├── router.ex           # HTTP route definitions
│   │   ├── endpoint.ex         # Phoenix HTTP endpoint
│   │   ├── controllers/        # HTTP request handlers
│   │   ├── components/         # Reusable UI components
│   │   └── templates/          # EEx HTML templates
│   ├── hackflare.ex            # Top-level application module
│   └── hackflare_web.ex        # Web layer configuration
├── config/
│   ├── config.exs              # General configuration
│   ├── dev.exs                 # Development-specific config
│   ├── prod.exs                # Production-specific config
│   ├── test.exs                # Test configuration
│   └── runtime.exs             # Runtime configuration (deployment)
├── native/core/                # Rust DNS engine
│   └── src/
│       ├── lib.rs              # Rust crate entry
│       ├── nifs.rs             # Elixir NIF bridge
│       ├── dns/                # DNS protocol implementation
│       └── ns/                 # Nameserver implementation
├── test/                       # Elixir test suite
├── priv/                       # Private static assets
└── mix.exs                     # Mix project definition
```

## Core Modules

### Application Bootstrap

**`Hackflare.Application`** - OTP Application supervision tree
- Starts database connection pool (`Hackflare.Repo`)
- Initializes telemetry and observability
- Starts DNS cluster for distributed deployments
- Launches nameserver process
- Starts Phoenix PubSub
- Starts HTTP endpoint

### Database

**`Hackflare.Repo`** - Ecto repository for PostgreSQL
- All database queries go through this module
- Handles migrations and schema definitions
- Connection pooling is configured in compile-time and runtime configs

### DNS Engine

**`Hackflare.Nameserver`** - GenServer managing DNS nameserver
- Bridges Elixir with Rust DNS implementation
- Initialized in supervision tree on startup
- Listens for DNS queries on configured bind address/port

**`Hackflare.Native`** - Rust NIF bindings
- Provides interface to high-performance Rust DNS engine
- Functions for zone management (create, delete, list zones)
- Functions for record operations (add, remove, find records)
- Query handling and nameserver control
- All functions are non-blocking

### Email

**`Hackflare.Mailer`** - Transactional email service
- Uses Swoosh mailer library
- Local mailbox preview in development (`/dev/mailbox`)
- Configurable adapters for production (SendGrid, Mailgun, AWS SES, etc.)

### Web Layer

**`HackflareWeb`** - Web layer configuration
- Defines macros for `use HackflareWeb, :controller`, `:live_view`, `:html`, etc.
- Centralizes imports, aliases, and setup for all web components
- Enables consistent structure across controllers, views, and components

**`HackflareWeb.Router`** - HTTP route definitions
- Defines request pipelines (`:browser` for HTML, `:api` for JSON)
- Routes requests to controllers and live views
- Development-only routes for dashboard and mailbox preview

**`HackflareWeb.PageController`** - Basic page handler
- Currently handles home page request
- Can be extended for other static pages

## Architecture Patterns

### Context-Based Design

The application follows Elixir/Phoenix conventions with contexts separating business logic:
- Database access through `Hackflare.Repo`
- Business logic in context modules (to be created)
- Web handling in controllers and live views

### Rust Integration

Performance-critical DNS operations are implemented in Rust:
- Compiled to native code via Rustler
- Called as NIFs (Native Implemented Functions) from Elixir
- Zero-copy query handling for maximum throughput
- Lives in `native/core/` directory

### GenServer for Long-Running Services

The nameserver runs as a GenServer:
- Started by the supervision tree
- Supervised and automatically restarted on failure
- Maintains state (the native DNS manager)
- Fault-tolerant design

## Configuration

### Development

Run with:
```bash
iex -S mix phx.server
```

Configuration files:
- `config/config.exs` - Base config
- `config/dev.exs` - Development overrides
- `config/runtime.exs` - Runtime environment-loaded config

Database setup:
```bash
mix ecto.create    # Create database
mix ecto.migrate   # Run migrations
```

### Production

Configuration via `config/runtime.exs` and environment variables:
- `DATABASE_URL` - PostgreSQL connection string
- `SECRET_KEY_BASE` - Encryption key for sessions
- `PHX_SERVER` - Set to "true" to start HTTP server

Built and deployed via Docker:
```bash
docker build -t hackflare .
docker run -p 4000:4000 -p 53:53/udp hackflare
```

## Key Dependencies

- **Phoenix** (~1.8.5) - Web framework
- **Phoenix LiveView** (~1.1.0) - Real-time UI
- **Ecto** (~3.10) - Database abstraction
- **Postgrex** - PostgreSQL adapter
- **Rustler** (~0.37.3) - Elixir/Rust integration
- **Swoosh** (~1.16) - Email handling
- **esbuild** (~0.10) - JavaScript bundler
- **Tailwind** (~0.3) - CSS framework
- **DNS Cluster** (~0.2.0) - Distributed DNS support

## Testing

Run the test suite:
```bash
mix test
```

Run with coverage:
```bash
mix test --cover
```

Run specific test file:
```bash
mix test test/path/to/test.exs
```

## Development Commands

```bash
mix setup              # Install deps and setup project
mix phx.server         # Start development server
mix phx.routes         # List all routes
mix test               # Run tests
mix format             # Format code
mix deps.get           # Install dependencies
mix deps.update        # Update dependencies
mix ecto.gen.migration # Create database migration
mix ecto.migrate       # Run pending migrations
```

## Debugging

### IEx Console in Development

```bash
iex -S mix phx.server
```

Helpful IEx commands:
```elixir
# Reload modules
recompile()

# Get all routes
HackflareWeb.Router.__routes__()

# Access the repo
Hackflare.Repo.all(Schema)

# Test mailer
Hackflare.Mailer.deliver(email)
```

### LiveDashboard

Available at `/dev/dashboard` in development for:
- Memory usage
- Process information
- Application metrics
- Ecto query performance

### Swoosh Mailbox

Available at `/dev/mailbox` to preview sent emails in development.

## Code Organization Best Practices

1. **Contexts** - Group related business logic in modules
2. **Schemas** - Define Ecto schemas for data structures
3. **Migrations** - Use Ecto migrations for database changes
4. **Controllers** - Keep minimal, delegate to contexts
5. **Views** - Define presentation logic, not business logic
6. **Components** - Reusable UI bits as Phoenix components
7. **Tests** - Test contexts thoroughly
8. **Documentation** - Use @moduledoc and @doc consistently

## Related Documentation

- [Phoenix Documentation](https://hexdocs.pm/phoenix)
- [Elixir Language](https://elixir-lang.org/docs.html)
- [Ecto Guide](https://hexdocs.pm/ecto/Ecto.html)
- [Phoenix LiveView](https://hexdocs.pm/phoenix_live_view)
- [Rustler](https://hexdocs.pm/rustler)

## Future Development

Features planned for implementation:
- [ ] Complete DNS system (zones, records, migrations)
- [ ] Authentication system (HackClub auth integration)
- [ ] Domain management interface
- [ ] Email routing and sending
- [ ] Performance monitoring
- [ ] Custom DNS records (SRV, TXT, etc.)
- [ ] Caching layer
- [ ] Load balancing
- [ ] DDoS protection
