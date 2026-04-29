defmodule Hackflare.AccountsAdminTest do
  use ExUnit.Case, async: false

  alias Hackflare.Accounts
  alias Hackflare.Accounts.User

  setup do
    previous_admin_emails = Application.get_env(:hackflare, :admin_emails, [])

    on_exit(fn ->
      Application.put_env(:hackflare, :admin_emails, previous_admin_emails)
    end)

    :ok
  end

  test "admin_user? returns true for explicit admins" do
    assert Accounts.admin_user?(%User{is_admin: true})
  end

  test "admin_user? returns true for allowlisted emails" do
    Application.put_env(:hackflare, :admin_emails, ["root@example.com"])

    assert Accounts.admin_user?(%User{email: "root@example.com"})
    refute Accounts.admin_user?(%User{email: "member@example.com"})
  end
end
