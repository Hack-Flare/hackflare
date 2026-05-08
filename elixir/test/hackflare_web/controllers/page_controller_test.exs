defmodule HackflareWeb.PageControllerTest do
  use HackflareWeb.ConnCase

  test "GET /", %{conn: conn} do
    conn = get(conn, ~p"/")
    assert html_response(conn, 200) =~ "HackFlare"
    assert html_response(conn, 200) =~ "Built by Hack Clubbers"
  end

  test "GET /health", %{conn: conn} do
    conn = get(conn, ~p"/health")

    assert response(conn, 200) == "ok"
  end
end
