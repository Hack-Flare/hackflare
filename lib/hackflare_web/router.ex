defmodule HackflareWeb.Router do
  @moduledoc """
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
  """
  use HackflareWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {HackflareWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  pipeline :require_authenticated do
    plug HackflareWeb.Plugs.RequireAuthenticated
  end

  pipeline :require_admin do
    plug HackflareWeb.Plugs.RequireAdmin
  end

  scope "/", HackflareWeb do
    pipe_through :browser

    get "/", PageController, :home
    get "/health", PageController, :health
    get "/docs", PageController, :docs_redirect
  end

  scope "/", HackflareWeb do
    pipe_through [:browser, :require_authenticated]

    get "/dash", DashController, :home
    get "/dash/domains", DashController, :domains
    get "/dash/settings", DashController, :settings
    get "/dash/analytics", DashController, :analytics
    get "/dash/notifications", DashController, :notifications
    get "/dash/help", DashController, :help
    post "/dash/help", DashController, :submit_help
  end

  scope "/admin", HackflareWeb do
    pipe_through [:browser, :require_authenticated, :require_admin]

    get "/", AdminController, :index
    post "/settings", AdminController, :update
  end

  scope "/auth", HackflareWeb do
    pipe_through :browser

    get "/request", AuthController, :request
    get "/callback", AuthController, :callback
    delete "/logout", AuthController, :logout
    post "/logout", AuthController, :logout
  end

  # Other scopes may use custom stacks.
  # scope "/api", HackflareWeb do
  #   pipe_through :api
  # end

  # Enable LiveDashboard and Swoosh mailbox preview in development
  if Application.compile_env(:hackflare, :dev_routes) do
    # If you want to use the LiveDashboard in production, you should put
    # it behind authentication and allow only admins to access it.
    # If your application does not have an admins-only section yet,
    # you can use Plug.BasicAuth to set up some basic authentication
    # as long as you are also using SSL (which you should anyway).
    import Phoenix.LiveDashboard.Router

    scope "/dev" do
      pipe_through :browser

      live_dashboard "/dashboard", metrics: HackflareWeb.Telemetry
      forward "/mailbox", Plug.Swoosh.MailboxPreview
    end
  end
end
