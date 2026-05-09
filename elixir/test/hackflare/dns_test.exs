defmodule Hackflare.DNSTest do
  use ExUnit.Case, async: false

  alias Hackflare.Accounts.User
  alias Hackflare.DNS
  alias Hackflare.DNS.Zone
  alias Hackflare.Repo
  import Ecto.Query

  setup do
    # Clean up zones before each test
    Repo.delete_all(from(u in User))
    Repo.delete_all(from(z in Zone))

    user =
      %User{}
      |> User.changeset(%{
        slack_id: "slack-#{System.unique_integer([:positive])}",
        email: "user-#{System.unique_integer([:positive])}@example.com",
        name: "Test User",
        verification_status: "verified",
        ysws_eligible: false,
        is_admin: false
      })
      |> Repo.insert!()

    {:ok, user: user}
  end

  describe "delete_zone/2" do
    test "deletes unverified zones (not in DNS manager)", %{user: user} do
      zone_name = "unverified.test"

      # Create an unverified zone
      assert {:ok, ^zone_name} = DNS.create_zone(zone_name, "root", user)

      # Verify it exists in DB but not verified
      zone = Repo.get_by(Zone, name: zone_name)
      assert zone
      assert zone.ns_verified == false

      # Should be able to delete unverified zone
      assert {:ok, ^zone_name} = DNS.delete_zone(zone_name, user)

      # Zone should be gone from DB
      refute Repo.get_by(Zone, name: zone_name)
    end

    test "keeps zones scoped to the owning user", %{user: user} do
      owned_zone = "owned.test"
      other_zone = "other.test"

      assert {:ok, ^owned_zone} = DNS.create_zone(owned_zone, "root", user)
      assert {:error, :owner_required} = DNS.create_zone(other_zone)

      owned_zone_row = Repo.get_by!(Zone, name: owned_zone)

      assert owned_zone_row.user_id == user.id

      assert {:ok, ^owned_zone} = DNS.delete_zone(owned_zone, user)

      assert {:error, :owner_required} = DNS.create_zone(other_zone)
      assert {:error, :zone_not_found} = DNS.delete_zone(other_zone, user)

      refute Repo.get_by(Zone, name: owned_zone)
      refute Repo.get_by(Zone, name: other_zone)
    end

    test "returns error for non-existent zones", %{user: user} do
      assert {:error, :zone_not_found} = DNS.delete_zone("nonexistent.test", user)
    end

    test "returns error for invalid zone names" do
      assert {:error, :invalid_zone_name} = DNS.delete_zone(nil)
      assert {:error, :invalid_zone_name} = DNS.delete_zone(123)
    end
  end

  describe "create_zone/3" do
    test "requires an owner" do
      assert {:error, :owner_required} = DNS.create_zone("orphan.test")
      assert {:error, :owner_required} = DNS.create_zone("orphan.test", "root")
    end

    test "returns conflict when zone already exists", %{user: user} do
      zone_name = "duplicate.test"

      assert {:ok, ^zone_name} = DNS.create_zone(zone_name, "root", user)
      assert {:error, :zone_already_exists} = DNS.create_zone(zone_name, "root", user)
    end
  end

  describe "owner-required mutating APIs" do
    test "delete_zone/1 requires owner" do
      assert {:error, :owner_required} = DNS.delete_zone("example.test")
    end

    test "add_record/5 requires owner" do
      assert {:error, :owner_required} = DNS.add_record("example.test", "@", "A", 300, "1.1.1.1")
    end

    test "remove_record/3 requires owner" do
      assert {:error, :owner_required} = DNS.remove_record("example.test", "@", "A")
    end

    test "verify_zone_nameservers/1 requires owner" do
      assert {:error, :owner_required} = DNS.verify_zone_nameservers("example.test")
    end
  end
end
