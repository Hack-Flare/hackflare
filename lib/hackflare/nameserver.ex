defmodule Hackflare.Nameserver do
  use GenServer

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  @impl true
  def init(_) do
    # create manager and start nameserver via native NIF
    config = Application.get_env(:hackflare, :dns)
    mgr = Hackflare.Native.manager_new()
    # start nameserver in background
    _ = Hackflare.Native.manager_start_nameserver(mgr, config.bind, config.port)
    {:ok, %{manager: mgr}}
  end
end
