defmodule Hackflare.Repo.Migrations.AddNsVerifiedToDnsZones do
  use Ecto.Migration

  def change do
    alter table(:dns_zones) do
      add :ns_verified, :boolean, null: false, default: false
    end
  end
end
