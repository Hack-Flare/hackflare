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

  test "POST /dash/domains/records/delete blocks edits until nameservers are verified", %{
    conn: conn,
    owner: owner
  } do
    conn =
      conn
      |> init_test_session(user_id: owner.id)
      |> post(~p"/dash/domains/records/delete", %{
        "zone_name" => "owned.example",
        "record_name" => "@",
        "record_type" => "A"
      })

    assert redirected_to(conn) == ~p"/dash/domains?zone_name=owned.example"
    assert Phoenix.Flash.get(conn.assigns.flash, :error) =~ "Verify nameservers"
  end

  test "POST /dash/domains/records/update blocks edits until nameservers are verified", %{
    conn: conn,
    owner: owner
  } do
    conn =
      conn
      |> init_test_session(user_id: owner.id)
      |> post(~p"/dash/domains/records/update", %{
        "zone_name" => "owned.example",
        "old_record_name" => "@",
        "old_record_type" => "A",
        "record_name" => "www",
        "record_type" => "A",
        "record_data" => "1.1.1.1",
        "ttl" => "300"
      })

    assert redirected_to(conn) == ~p"/dash/domains?zone_name=owned.example"
    assert Phoenix.Flash.get(conn.assigns.flash, :error) =~ "Verify nameservers"
  end
end
