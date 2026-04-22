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
  Renders the home page using the Layouts.home component.

  Delegates to the Layouts module for rendering the homepage with standard
  navigation and layout structure.

  ## Parameters

    - `"home.html"` - Template name
    - `assigns` - Assigns map containing variables for the template

  ## Returns

    Rendered HTML output
  """
  def render("home.html", assigns) do
    HackflareWeb.Layouts.home(assigns)
  end

  @doc """
  Renders the dashboard page using the Layouts.dashboard component.

  Delegates to the Layouts module for rendering the dashboard view.
  """
  def render("dashboard.html", assigns) do
    HackflareWeb.Layouts.dashboard(assigns)
  end
end
