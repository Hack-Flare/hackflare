defmodule HackflareWeb.Plugs.RequireAuthenticated do
  @moduledoc """
  Plug to ensure a user is signed in via `:user_id` in the session.

  If no signed-in user is found, redirects to the landing page with an
  error flash and halts the connection.
  """
  import Plug.Conn
  import Phoenix.Controller

  def init(opts), do: opts

  def call(conn, _opts) do
    case get_session(conn, :user_id) do
      nil ->
        conn
        |> redirect(to: "/auth/request")
        |> halt()

      _user_id ->
        conn
    end
  end
end
