defmodule HackflareWeb.PageController do
  @moduledoc """
  Controller for static pages in the Hackflare web application.

  This controller handles basic page requests such as the home page.
  For dynamic, real-time pages, use LiveView modules instead.
  """
  use HackflareWeb, :controller

  @doc """
  Render the home page.

  This is the landing page for users visiting the Hackflare application.
  It displays information about the service and entry points for authentication.

  ## Parameters

    - `conn` - The Plug.Conn connection struct
    - `_params` - Query parameters (unused)

  ## Returns

  Renders the `home.html` template.
  """
  def home(conn, _params) do
    render(conn, :home)
  end

  @doc """
  Return a lightweight response for health checks.
  """
  def health(conn, _params) do
    text(conn, "ok")
  end

  @doc """
  Redirect `/docs` to the generated documentation index page.
  """
  def docs_redirect(conn, _params) do
    redirect(conn, to: "/docs/index.html")
  end
end
