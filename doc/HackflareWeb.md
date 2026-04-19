# `HackflareWeb`

The entrypoint for defining your web interface and Phoenix components.

This module provides the common setup and helper functions for all web-layer components,
including controllers, live views, templates, and components. It follows Phoenix conventions
for organizing HTTP handlers, real-time features, and UI components.

## Usage

Import this module in your web components:

    use HackflareWeb, :controller
    use HackflareWeb, :live_view
    use HackflareWeb, :html
    use HackflareWeb, :router

## Features

- **Controllers** - Handle HTTP requests and responses
- **LiveViews** - Real-time interactive UI with WebSocket support
- **Components** - Reusable UI elements built with Phoenix.Component
- **Templates** - HTML5 templates with EEx templating
- **Router** - HTTP routing and pipeline configuration

## Architecture

The web layer is layered as:
1. **Endpoint** (`HackflareWeb.Endpoint`) - HTTP entry point
2. **Router** (`HackflareWeb.Router`) - Route definitions and pipelines
3. **Controllers** - Request handlers
4. **Views/Components** - Response rendering
5. **Templates** - HTML templates with interpolation

Static files are served from `priv/static/` and include assets built by esbuild (JS)
and Tailwind (CSS).

# `__using__`
*macro* 

When used, dispatch to the appropriate controller/live_view/etc.

# `channel`

Returns the setup for Phoenix channels.

When used with `use HackflareWeb, :channel`, provides the base channel module
for WebSocket/long-polling connections.

## Examples

    use HackflareWeb, :channel

    def join("room:lobby", _message, socket) do
      {:ok, socket}
    end

# `controller`

Returns the setup for controllers.

When used with `use HackflareWeb, :controller`, provides common imports and
configuration for HTTP request handlers supporting both HTML and JSON rendering.

## Examples

    use HackflareWeb, :controller

    def action(conn, _params) do
      render(conn, :index, items: get_items())
    end

# `html`

Returns the setup for HTML/component modules.

When used with `use HackflareWeb, :html`, provides imports for building
reusable HTML components and helpers.

## Examples

    use HackflareWeb, :html

    def my_component(assigns) do
      ~H"<div><%= @content %></div>"
    end

# `live_component`

Returns the setup for LiveComponent modules.

When used with `use HackflareWeb, :live_component`, provides configuration
for reusable stateful components within LiveView pages.

## Examples

    use HackflareWeb, :live_component

    def render(assigns) do
      ~H"<button phx-click="click"><%= @label %></button>"
    end

# `live_view`

Returns the setup for LiveView modules.

When used with `use HackflareWeb, :live_view`, provides common configuration
for real-time interactive UI components with WebSocket support.

## Examples

    use HackflareWeb, :live_view

    def render(assigns) do
      ~H"""
      <div>
        <button phx-click="increment">
          Count: <%= @count %>
        </button>
      </div>
      """
    end

# `router`

Returns the setup for routers.

When used with `use HackflareWeb, :router`, provides common imports and
configuration for HTTP routers.

## Examples

    use HackflareWeb, :router

    scope "/", MyApp do
      pipe_through :browser
      get "/", PageController, :index
    end

# `static_paths`

Returns the list of static file paths that should be served by the application.

These paths are served from the `priv/static` directory and include:
- `assets/` - Compiled JavaScript and CSS
- `fonts/` - Custom fonts
- `images/` - Image assets
- Standard files: favicon.ico, robots.txt

## Returns

  List of static file paths as strings

# `verified_routes`

Returns the setup for verified routes (the ~p sigil).

When included in modules, enables the use of the verified routes sigil (~p)
for compile-time verified internal links. This prevents broken routes from
compiling.

## Examples

    ~p\"/posts/#{post.id}\"
    ~p\"/users?sort=name\"

---

*Consult [api-reference.md](api-reference.md) for complete listing*
