defmodule HackflareWeb.AdminHTML do
  @moduledoc """
  This module contains pages rendered by AdminController.
  """

  use HackflareWeb, :html

  @doc """
  Renders the admin panel using the dashboard layout.
  """
  def render("index.html", assigns) do
    HackflareWeb.Layouts.dashboard(assigns)
  end
end
