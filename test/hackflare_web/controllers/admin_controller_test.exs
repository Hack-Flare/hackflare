defmodule HackflareWeb.AdminControllerTest do
  use HackflareWeb.ConnCase

  test "GET /admin requires authentication", %{conn: conn} do
    conn = get(conn, ~p"/admin")

    assert redirected_to(conn) == "/auth/request"

    assert Phoenix.Flash.get(conn.assigns.flash, :error) ==
             "You need to be signed in to access the dashboard"
  end
end
