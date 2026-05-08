defmodule HackflareWeb.Plugs.RequireAdmin do
  @moduledoc """
  Plug to ensure the signed-in user has admin access.
  """

  import Plug.Conn
  import Phoenix.Controller

  alias Hackflare.Accounts

  def init(opts), do: opts

  def call(conn, _opts) do
    with user_id when not is_nil(user_id) <- get_session(conn, :user_id),
         %{} = user <- Accounts.get_user(user_id),
         true <- Accounts.admin_user?(user) do
      conn
    else
      _ ->
        conn
        |> put_flash(:error, "Admin access required.")
        |> redirect(to: "/dash")
        |> halt()
    end
  end
end
