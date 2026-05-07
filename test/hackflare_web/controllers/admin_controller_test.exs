defmodule HackflareWeb.AdminControllerTest do
  use HackflareWeb.ConnCase

  alias Hackflare.Accounts.User
  alias Hackflare.DNS
  alias Hackflare.Repo

  setup do
    Repo.delete_all(User)

    admin =
      %User{}
      |> User.changeset(%{
        slack_id: "admin-#{System.unique_integer([:positive])}",
        email: "admin-#{System.unique_integer([:positive])}@example.com",
        name: "Admin User",
        verification_status: "verified",
        ysws_eligible: false,
        is_admin: true
      })
      |> Repo.insert!()

    other_user =
      %User{}
      |> User.changeset(%{
        slack_id: "owner-#{System.unique_integer([:positive])}",
        email: "owner-#{System.unique_integer([:positive])}@example.com",
        name: "Owner User",
        verification_status: "verified",
        ysws_eligible: false,
        is_admin: false
      })
      |> Repo.insert!()

    DNS.create_zone("admin-owned.example", "root", admin)
    DNS.create_zone("owner.example", "root", other_user)

    {:ok, admin: admin}
  end

  test "GET /admin requires authentication", %{conn: conn} do
    conn = get(conn, ~p"/admin")

    assert redirected_to(conn) == "/auth/request"
  end

  test "GET /admin shows the admin domain search menu", %{conn: conn, admin: admin} do
    conn =
      conn
      |> init_test_session(user_id: admin.id)
      |> get(~p"/admin")

    html = html_response(conn, 200)

    assert html =~ "Open Search"
    assert html =~ "admin-owned.example"
    assert html =~ "owner.example"
    assert html =~ "Edit"
  end
end
