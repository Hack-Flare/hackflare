defmodule HackflareWeb.DashControllerTest do
  use HackflareWeb.ConnCase

  alias Hackflare.Accounts.User
  alias Hackflare.DNS
  alias Hackflare.Repo

  setup do
    Repo.delete_all(User)

    owner =
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

    other_user =
      %User{}
      |> User.changeset(%{
        slack_id: "other-#{System.unique_integer([:positive])}",
        email: "other-#{System.unique_integer([:positive])}@example.com",
        name: "Other User",
        verification_status: "verified",
        ysws_eligible: false,
        is_admin: false
      })
      |> Repo.insert!()

    DNS.create_zone("owned.example", "root", owner)
    DNS.create_zone("hidden.example", "root", other_user)

    {:ok, owner: owner, other_user: other_user}
  end

  test "GET /dash/domains only shows the signed-in user's domains", %{conn: conn, owner: owner} do
    conn =
      conn
      |> init_test_session(user_id: owner.id)
      |> get(~p"/dash/domains")

    html = html_response(conn, 200)

    assert html =~ "owned.example"
    refute html =~ "hidden.example"
  end
end
