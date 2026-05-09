defmodule HackflareWeb.DashHTML do
  @moduledoc """
  This module contains pages rendered by DashController.
  """
  use HackflareWeb, :html

  @doc """
  Renders the dashboard page using the Layouts.dashboard component.
  """
  def render("dashboard.html", assigns) do
    HackflareWeb.Layouts.dashboard(assigns)
  end
end
