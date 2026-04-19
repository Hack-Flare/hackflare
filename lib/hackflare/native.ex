defmodule Hackflare.Native do
  use Rustler, otp_app: :hackflare, crate: "core"

  # NIF stubs (called if native library is not loaded)
  def manager_new(), do: :erlang.nif_error(:nif_not_loaded)
  def manager_create_zone(_mgr, _name), do: :erlang.nif_error(:nif_not_loaded)
  def manager_delete_zone(_mgr, _name), do: :erlang.nif_error(:nif_not_loaded)
  def manager_add_record(_mgr, _zone, _name, _type, _ttl, _data), do: :erlang.nif_error(:nif_not_loaded)
  def manager_remove_record(_mgr, _zone, _name, _type), do: :erlang.nif_error(:nif_not_loaded)
  def manager_list_zones(_mgr), do: :erlang.nif_error(:nif_not_loaded)
  def manager_find_records(_mgr, _name, _type \\ nil), do: :erlang.nif_error(:nif_not_loaded)
  def engine_handle_query(_mgr, _query), do: :erlang.nif_error(:nif_not_loaded)
  def manager_start_nameserver(_mgr, _bind, _port), do: :erlang.nif_error(:nif_not_loaded)
end
