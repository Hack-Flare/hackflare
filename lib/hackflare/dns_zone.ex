defmodule Hackflare.DNS.Zone do
  use Ecto.Schema
  import Ecto.Changeset

  schema "dns_zones" do
    belongs_to(:user, Hackflare.Accounts.User)
    field(:name, :string)
    field(:type, :string, default: "root")
    field(:ns_verified, :boolean, default: false)
    has_many(:records, Hackflare.DNS.Record, foreign_key: :zone_id)

    timestamps()
  end

  def changeset(zone, attrs) do
    zone
    |> cast(attrs, [:user_id, :name, :type, :ns_verified])
    |> validate_required([:user_id, :name])
    |> unique_constraint(:name)
  end
end
