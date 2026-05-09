defmodule Hackflare.Application do
  @moduledoc """
  The OTP Application entry point and supervision tree for Hackflare.

  This module defines the supervision tree that starts all essential services
  for the Hackflare application. It's called by the Elixir runtime on startup
  and supervises all child processes needed for the application to run.

  ## Supervision Tree

  The following services are started and supervised:

  - `Hackflare.Repo` - PostgreSQL database connection pool
  - `HackflareWeb.Telemetry` - Observability and metrics collection
  - `DNSCluster` - DNS cluster management for distributed deployments
  - `Hackflare.Nameserver` - DNS nameserver process handling DNS queries
  - `Hackflare.PubSub` - Phoenix PubSub for real-time communication
  - `HackflareWeb.Endpoint` - HTTP endpoint serving the web application

  ## Strategy

  Uses `:one_for_one` supervision strategy, meaning if any child process crashes,
  only that child is restarted, not the entire tree.
  """

  use Application

  @impl true
  @doc """
  Starts the Hackflare application and its supervision tree.

  This callback is invoked by the Elixir runtime when the application is started.
  It creates the supervision tree with all child processes and returns the root
  supervisor's PID.

  ## Parameters

    - `_type` - Application start type (`:normal`, `:transient`, or `:temporary`)
    - `_args` - Arguments passed to the application (usually from mix/config)

  ## Returns

    - `{:ok, pid}` - PID of the root supervisor
    - `{:error, reason}` - If supervision tree startup fails
  """
  def start(_type, _args) do
    children = [
      Hackflare.Repo,
      HackflareWeb.Telemetry,
      {DNSCluster, query: Application.get_env(:hackflare, :dns_cluster_query) || :ignore},
      Hackflare.Nameserver,
      {Phoenix.PubSub, name: Hackflare.PubSub},
      # Start a worker by calling: Hackflare.Worker.start_link(arg)
      # {Hackflare.Worker, arg},
      # Start to serve requests, typically the last entry
      HackflareWeb.Endpoint
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Hackflare.Supervisor]
    Supervisor.start_link(children, opts)
  end

  @impl true
  @doc """
  Called when application configuration changes at runtime.

  This callback is invoked when the application configuration is updated during
  runtime (typically in development with hot reloading). It propagates configuration
  changes to the Phoenix endpoint.

  ## Parameters

    - `changed` - Keyword list of changed configs
    - `_new` - New configuration values (unused)
    - `removed` - List of removed configuration keys

  ## Returns
    `:ok` - Configuration change was applied successfully
  """
  def config_change(changed, _new, removed) do
    HackflareWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
