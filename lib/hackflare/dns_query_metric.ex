defmodule Hackflare.DnsQueryMetric do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :integer, autogenerate: false}
  schema "dns_query_metrics" do
    field(:udp_count, :integer, default: 0)
    field(:tcp_count, :integer, default: 0)

    timestamps()
  end

  def changeset(metric, attrs) do
    metric
    |> cast(attrs, [:udp_count, :tcp_count])
    |> validate_required([:udp_count, :tcp_count])
  end
end
