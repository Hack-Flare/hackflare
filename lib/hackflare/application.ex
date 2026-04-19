defmodule Hackflare.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
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

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    HackflareWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
