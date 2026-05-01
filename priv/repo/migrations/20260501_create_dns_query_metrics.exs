defmodule Hackflare.Repo.Migrations.CreateDnsQueryMetrics do
  use Ecto.Migration

  def change do
    create table(:dns_query_metrics) do
      add :udp_count, :bigint, null: false, default: 0
      add :tcp_count, :bigint, null: false, default: 0

      timestamps()
    end

    execute("INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at) VALUES (1, 0, 0, now(), now())")
    create unique_index(:dns_query_metrics, [:id])
  end
end
