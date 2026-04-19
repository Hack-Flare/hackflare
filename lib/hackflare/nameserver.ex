defmodule Hackflare.Nameserver do
  use GenServer

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  @impl true
  def init(_) do
    # create manager and start nameserver via native NIF
    mgr = Hackflare.Native.manager_new()
    bind = Application.get_env(:hackflare, :dns_bind, "0.0.0.0")
    port = Application.get_env(:hackflare, :dns_port, 53)
    # start nameserver in background
    _ = Hackflare.Native.manager_start_nameserver(mgr, bind, port)
    {:ok, %{manager: mgr}}
  end
end
