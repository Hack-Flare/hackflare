defmodule HackflareWeb.DashController do
  @moduledoc """
  Controller for authenticated dashboard views.
  """
  use HackflareWeb, :controller

  def home(conn, _params) do
    render(conn, :dashboard, current_view: :home)
  end

  def domains(conn, _params) do
    render(conn, :dashboard, current_view: :domains)
  end

  def settings(conn, _params) do
    render(conn, :dashboard, current_view: :settings)
  end

  def analytics(conn, _params) do
    render(conn, :dashboard, current_view: :analytics)
  end

  def notifications(conn, _params) do
    render(conn, :dashboard, current_view: :notifications)
  end

  def help(conn, _params) do
    render(conn, :dashboard, current_view: :help)
  end
end
