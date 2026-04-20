# `HackflareWeb.PageController`

Controller for static pages in the Hackflare web application.

This controller handles basic page requests such as the home page.
For dynamic, real-time pages, use LiveView modules instead.

# `docs_redirect`

Redirect `/docs` to the generated documentation index page.

# `home`

Render the home page.

This is the landing page for users visiting the Hackflare application.
It displays information about the service and entry points for authentication.

## Parameters

  - `conn` - The Plug.Conn connection struct
  - `_params` - Query parameters (unused)

## Returns

Renders the `home.html` template.

---

*Consult [api-reference.md](api-reference.md) for complete listing*
