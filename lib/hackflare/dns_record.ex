defmodule Hackflare.DNS.Record do
  use Ecto.Schema
  import Ecto.Changeset

  schema "dns_records" do
    field(:name, :string)
    field(:rtype, :string)
    field(:ttl, :integer)
    field(:data, :string)
    belongs_to(:zone, Hackflare.DNS.Zone, foreign_key: :zone_id)

    timestamps()
  end

  def changeset(record, attrs) do
    record
    |> cast(attrs, [:zone_id, :name, :rtype, :ttl, :data])
    |> validate_required([:zone_id, :name, :rtype, :ttl, :data])
  end
end
