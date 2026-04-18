defmodule Hackflare.Native do
  use Rustler,
    otp_app: :hackflare,
    crate: "core"

  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
end