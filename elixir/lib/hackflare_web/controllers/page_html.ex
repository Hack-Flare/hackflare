defmodule HackflareWeb.PageHTML do
  @moduledoc """
  This module contains pages rendered by PageController.

  See the `page_html` directory for all templates available.
  """
  use HackflareWeb, :html
  # The homepage is provided by the `Layouts` component directly.
  # We keep the embed_templates call for future page templates.
  embed_templates "page_html/*"

  @doc """
  Renders a page by `template` name.

  Supported templates:

    - "home.html" — Renders the home page using `Layouts.home`.
    - "dashboard.html" — Renders the dashboard page using `Layouts.dashboard`.

  ## Parameters

    - `template` - Template name (string)
    - `assigns` - Assigns map containing variables for the template

  ## Returns

    Rendered HTML output
  """
  def render(template, assigns)

  def render("home.html", assigns) do
    HackflareWeb.Layouts.home(assigns)
  end

  def render("dashboard.html", assigns) do
    HackflareWeb.Layouts.dashboard(assigns)
  end
end
