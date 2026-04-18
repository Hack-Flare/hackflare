defmodule HackflareWeb.PageHTML do
  @moduledoc """
  This module contains pages rendered by PageController.

  See the `page_html` directory for all templates available.
  """
  use HackflareWeb, :html
  # The homepage is provided by the `Layouts` component directly.
  # We keep the embed_templates call for future page templates.
  embed_templates "page_html/*"

  def render("home.html", assigns) do
    HackflareWeb.Layouts.home(assigns)
  end
end
