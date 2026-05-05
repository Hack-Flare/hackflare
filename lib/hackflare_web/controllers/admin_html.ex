defmodule HackflareWeb.AdminHTML do
  @moduledoc """
  This module contains pages rendered by AdminController.
  """

  use HackflareWeb, :html

  embed_templates "admin_html/*"

  @doc """
  Renders the admin panel.
  """
  def render("index.html", assigns) do
    admin(assigns)
  end
end
