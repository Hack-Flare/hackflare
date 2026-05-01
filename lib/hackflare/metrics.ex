defmodule Hackflare.Metrics do
  @moduledoc """
  Persistent metrics helpers for DNS query counters.
  """
  alias Hackflare.DnsQueryMetric
  alias Hackflare.Repo

  def get_metrics do
    case Repo.get(DnsQueryMetric, 1) do
      nil -> %DnsQueryMetric{id: 1, udp_count: 0, tcp_count: 0}
      m -> m
    end
  end

  def inc_udp do
    sql = """
    INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at)
    VALUES (1, 1, 0, now(), now())
    ON CONFLICT (id) DO UPDATE SET udp_count = dns_query_metrics.udp_count + 1, updated_at = now();
    """

    {:ok, _} = Repo.query(sql)
    :ok
  end

  def inc_tcp do
    sql = """
    INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at)
    VALUES (1, 0, 1, now(), now())
    ON CONFLICT (id) DO UPDATE SET tcp_count = dns_query_metrics.tcp_count + 1, updated_at = now();
    """

    {:ok, _} = Repo.query(sql)
    :ok
  end
end
