defmodule Hackflare.DNSTest do
  use ExUnit.Case, async: false

  alias Hackflare.DNS
  alias Hackflare.DNS.Zone
  alias Hackflare.Repo
  import Ecto.Query

  setup do
    # Clean up zones before each test
    Repo.delete_all(from(z in Zone))
    :ok
  end

  describe "delete_zone/1" do
    test "deletes unverified zones (not in DNS manager)" do
      zone_name = "unverified.test"

      # Create an unverified zone
      assert {:ok, ^zone_name} = DNS.create_zone(zone_name)

      # Verify it exists in DB but not verified
      zone = Repo.get_by(Zone, name: zone_name)
      assert zone
      assert zone.ns_verified == false

      # Should be able to delete unverified zone
      assert {:ok, ^zone_name} = DNS.delete_zone(zone_name)

      # Zone should be gone from DB
      refute Repo.get_by(Zone, name: zone_name)
    end

    test "returns error for non-existent zones" do
      assert {:error, :zone_not_found} = DNS.delete_zone("nonexistent.test")
    end

    test "returns error for invalid zone names" do
      assert {:error, :invalid_zone_name} = DNS.delete_zone(nil)
      assert {:error, :invalid_zone_name} = DNS.delete_zone(123)
    end
  end
end
