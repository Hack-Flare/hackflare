defmodule Hackflare.Nameserver do
  @moduledoc """
  Manages the DNS nameserver process and native Rust integration.

  This GenServer acts as a bridge between the Elixir application and the Rust-based
  DNS nameserver implementation. It initializes and manages the lifetime of the native
  DNS manager, which handles all DNS query processing and zone management.

  ## Responsibilities

  - Create and maintain the native DNS manager instance
  - Start the nameserver listening on the configured bind address and port
  - Handle GenServer lifecycle (start, stop, restart)
  - Provide interface to DNS operations via `Hackflare.Native`

  ## Configuration

  The nameserver is configured via the `:dns` application environment variable,
  which should contain:

      config :hackflare,
        dns: [
          bind: "0.0.0.0",
          port: 53
        ]

  ## Native Integration

  This module uses Rustler NIF bindings in `Hackflare.Native` to communicate with
  the Rust nameserver implementation. The native manager handles the actual DNS
  protocol parsing, zone storage, and query resolution.
  """
  use GenServer

  @doc """
  Starts the DNS nameserver GenServer.

  This function is called by the supervisor to initialize the nameserver process.
  It registers the process under the module name so it can be called globally.

  ## Parameters

    - `_opts` - Supervisor options (unused)

  ## Returns

    - `{:ok, pid}` - PID of the started nameserver process
    - `{:error, reason}` - If startup fails
  """
  def start_link(_opts) do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  @impl true
  @doc false
  def init(_) do
    # create manager and start nameserver via native NIF
    config = Hackflare.Settings.dns_config()
    export_soa_config(Map.get(config, :soa, %{}))
    # Create manager and populate from DB
    mgr = load_manager_from_db()

    # start nameserver in background
    _ =
      Hackflare.Native.manager_start_nameserver(
        mgr,
        Map.get(config, :bind, "0.0.0.0"),
        Map.get(config, :port, 53)
      )

    {:ok, %{manager: mgr}}
  end

  @impl true
  def handle_call({:get_manager}, _from, state) do
    {:reply, state.manager, state}
  end

  @impl true
  def handle_info(_msg, state) do
    {:noreply, state}
  end

  @impl true
  def terminate(_reason, _state) do
    :ok
  end

  defp load_manager_from_db do
    zones = Hackflare.Repo.all(Hackflare.DNS.Zone) |> Hackflare.Repo.preload(:records)

    case zones do
      [] ->
        IO.puts("No zones in DB; creating fresh DNS manager")
        Hackflare.Native.manager_new()

      zones_list when is_list(zones_list) ->
        mgr = populate_manager_from_zones(zones_list)
        IO.puts("Loaded DNS zones from DB (#{length(zones_list)} zones)")
        mgr
    end
  rescue
    e in Postgrex.Error ->
      code =
        case e.postgres do
          m when is_map(m) -> Map.get(m, :code)
          _ -> nil
        end

      case code do
        :undefined_table ->
          IO.puts("DNS tables not migrated yet; creating fresh DNS manager")
          Hackflare.Native.manager_new()

        :invalid_catalog_name ->
          IO.puts("DNS database not created yet; creating fresh DNS manager")
          Hackflare.Native.manager_new()

        _ ->
          reraise e, __STACKTRACE__
      end
  end

  defp populate_manager_from_zones(zones) do
    mgr = Hackflare.Native.manager_new()

    Enum.each(zones, fn zone ->
      _ = Hackflare.Native.manager_create_zone(mgr, zone.name)
      add_zone_records_to_manager(mgr, zone)
    end)

    mgr
  end

  defp add_zone_records_to_manager(mgr, zone) do
    Enum.each(zone.records, fn rec ->
      _ =
        Hackflare.Native.manager_add_record(
          mgr,
          zone.name,
          rec.name,
          rec.rtype,
          rec.ttl,
          rec.data
        )
    end)
  end

  # File-based save/load removed; persistence handled by DB

  def restart do
    Supervisor.restart_child(Hackflare.Supervisor, __MODULE__)
  end

  defp export_soa_config(soa) when is_map(soa) do
    System.put_env("HACKFLARE_DNS_SOA_MNAME", Map.get(soa, :mname, "a.root-servers.net."))
    System.put_env("HACKFLARE_DNS_SOA_RNAME", Map.get(soa, :rname, "nstld.verisign-grs.com."))

    System.put_env(
      "HACKFLARE_DNS_SOA_SERIAL",
      Integer.to_string(Map.get(soa, :serial, 2_026_042_000))
    )

    System.put_env("HACKFLARE_DNS_SOA_REFRESH", Integer.to_string(Map.get(soa, :refresh, 1800)))
    System.put_env("HACKFLARE_DNS_SOA_RETRY", Integer.to_string(Map.get(soa, :retry, 900)))
    System.put_env("HACKFLARE_DNS_SOA_EXPIRE", Integer.to_string(Map.get(soa, :expire, 604_800)))
    System.put_env("HACKFLARE_DNS_SOA_MINIMUM", Integer.to_string(Map.get(soa, :minimum, 86_400)))
    System.put_env("HACKFLARE_DNS_SOA_TTL", Integer.to_string(Map.get(soa, :ttl, 86_400)))
  end

  defp export_soa_config(_), do: :ok
end
