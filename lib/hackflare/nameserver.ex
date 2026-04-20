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
    config = Application.get_env(:hackflare, :dns, %{})
    export_soa_config(Map.get(config, :soa, %{}))
    mgr = Hackflare.Native.manager_new()
    # start nameserver in background
    _ = Hackflare.Native.manager_start_nameserver(
      mgr,
      Map.get(config, :bind, "0.0.0.0"),
      Map.get(config, :port, 53)
    )
    {:ok, %{manager: mgr}}
  end

  defp export_soa_config(soa) when is_map(soa) do
    System.put_env("HACKFLARE_DNS_SOA_MNAME", Map.get(soa, :mname, "a.root-servers.net."))
    System.put_env("HACKFLARE_DNS_SOA_RNAME", Map.get(soa, :rname, "nstld.verisign-grs.com."))
    System.put_env("HACKFLARE_DNS_SOA_SERIAL", Integer.to_string(Map.get(soa, :serial, 2026042000)))
    System.put_env("HACKFLARE_DNS_SOA_REFRESH", Integer.to_string(Map.get(soa, :refresh, 1800)))
    System.put_env("HACKFLARE_DNS_SOA_RETRY", Integer.to_string(Map.get(soa, :retry, 900)))
    System.put_env("HACKFLARE_DNS_SOA_EXPIRE", Integer.to_string(Map.get(soa, :expire, 604_800)))
    System.put_env("HACKFLARE_DNS_SOA_MINIMUM", Integer.to_string(Map.get(soa, :minimum, 86_400)))
    System.put_env("HACKFLARE_DNS_SOA_TTL", Integer.to_string(Map.get(soa, :ttl, 86_400)))
  end

  defp export_soa_config(_), do: :ok
end
