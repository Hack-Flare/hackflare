defmodule HackflareWeb.AdminControllerTest do
  use HackflareWeb.ConnCase

  test "GET /admin requires authentication", %{conn: conn} do
    conn = get(conn, ~p"/admin")

    assert redirected_to(conn) == "/"

    assert Phoenix.Flash.get(conn.assigns.flash, :error) ==
             "You need to be signed in to access dash"
  end
end
