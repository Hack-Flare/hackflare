defmodule Hackflare.Repo.Migrations.CreateDnsZonesAndRecords do
  use Ecto.Migration

  def change do
    create table(:dns_zones) do
      add :name, :string, null: false
      add :type, :string, null: false, default: "root"
      add :ns_verified, :boolean, null: false, default: false

      timestamps()
    end

    create unique_index(:dns_zones, [:name])

    create table(:dns_records) do
      add :zone_id, references(:dns_zones, on_delete: :delete_all), null: false
      add :name, :string, null: false
      add :rtype, :string, null: false
      add :ttl, :integer, null: false, default: 300
      add :data, :text, null: false

      timestamps()
    end

    create index(:dns_records, [:zone_id])
  end
end
