# `HackflareWeb.Router`

HTTP Router for Hackflare web application.

This module defines all HTTP routes and request pipelines for the Hackflare
web interface. Routes are matched against incoming HTTP requests and dispatched
to the appropriate controller or live view handler.

## Pipelines

Pipelines are sequences of plugs that process requests before they reach handlers.

### :browser

Used for traditional HTML requests. Includes:
- CSRF protection
- Session handling
- LiveView flash support
- Secure headers

### :api

Used for JSON API requests. Includes:
- JSON content type handling
- No CSRF/session requirements

## Routes

Currently defined routes:
- `GET /` - Home page (PageController.home)
- `/dev/dashboard` - LiveDashboard (development only)
- `/dev/mailbox` - Email preview (development only)

## Development-only Routes

The `/dev` scope is only mounted when `dev_routes` is enabled in compile-time
configuration. This provides development tools:
- LiveDashboard for monitoring and debugging
- Swoosh mailbox preview for email testing

## Extending Routes

To add new routes:

    scope "/api/v1", HackflareWeb do
      pipe_through :api
      resources "/domains", DomainController
    end

# `api`

# `browser`

# `call`

Callback invoked by Plug on every request.

# `formatted_routes`

# `init`

Callback required by Plug that initializes the router
for serving web requests.

# `verified_route?`

---

*Consult [api-reference.md](api-reference.md) for complete listing*
