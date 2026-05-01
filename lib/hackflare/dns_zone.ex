defmodule Hackflare.DNS.Zone do
  use Ecto.Schema
  import Ecto.Changeset

  schema "dns_zones" do
    field(:name, :string)
    field(:type, :string, default: "root")
    field(:ns_verified, :boolean, default: false)
    has_many(:records, Hackflare.DNS.Record, foreign_key: :zone_id)

    timestamps()
  end

  def changeset(zone, attrs) do
    zone
    |> cast(attrs, [:name, :type, :ns_verified])
    |> validate_required([:name])
    |> unique_constraint(:name)
  end
end
