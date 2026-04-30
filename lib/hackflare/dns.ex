defmodule Hackflare.DNS do
  @moduledoc """
  DNS zone and record management context.

  Provides a high-level interface for managing DNS zones and records,
  wrapping the low-level Rust NIF calls from Hackflare.Native.
  """

  alias Hackflare.DNS.Record
  alias Hackflare.DNS.Zone
  alias Hackflare.Native
  alias Hackflare.Repo
  import Ecto.Query, only: [from: 2]

  @doc """
  Get the DNS manager instance.

  Returns the manager reference stored in the Nameserver GenServer.
  """
  def get_manager do
    case GenServer.call(Hackflare.Nameserver, {:get_manager}, 5000) do
      manager when is_reference(manager) -> {:ok, manager}
      nil -> {:error, :nameserver_not_started}
    end
  rescue
    _ -> {:error, :nameserver_not_started}
  end

  @doc """
  List all DNS zones.

  Returns a list of zone names as strings.
  """
  def list_zones do
    with {:ok, mgr} <- get_manager(),
         json when is_binary(json) <- Native.manager_list_zones(mgr) do
      decode_zones(json)
    else
      _ -> {:error, :failed_to_list_zones}
    end
  end

  defp decode_zones(json) do
    case Jason.decode(json) do
      {:ok, zones} when is_list(zones) -> {:ok, zones}
      _ -> {:error, :invalid_zones_response}
    end
  end

  @doc """
  Create a new DNS zone.

  Returns `{:ok, zone_name}` on success, `{:error, reason}` on failure.
  """
  def create_zone(zone_name) when is_binary(zone_name) do
    with {:ok, mgr} <- get_manager() do
      case Native.manager_create_zone(mgr, zone_name) do
        true ->
          # persist zone to DB if not exists
          %Zone{}
          |> Zone.changeset(%{name: zone_name})
          |> Repo.insert(on_conflict: :nothing)

          {:ok, zone_name}

        false ->
          {:error, :failed_to_create_zone}
      end
    end
  end

  def create_zone(_), do: {:error, :invalid_zone_name}

  def create_zone(zone_name, _zone_type) when is_binary(zone_name) do
    create_zone(zone_name)
  end

  @doc """
  Delete a DNS zone.

  Returns `{:ok, zone_name}` on success, `{:error, reason}` on failure.
  """
  def delete_zone(zone_name) when is_binary(zone_name) do
    with {:ok, mgr} <- get_manager() do
      case Native.manager_delete_zone(mgr, zone_name) do
        true ->
          # remove from DB as well
          from(z in Zone, where: z.name == ^zone_name) |> Repo.delete_all()
          {:ok, zone_name}

        false ->
          {:error, :zone_not_found}
      end
    end
  end

  def delete_zone(_), do: {:error, :invalid_zone_name}

  @doc """
  List all records in a zone, optionally filtered by name and/or type.

  Returns a list of record maps with keys: name, rtype, ttl, data
  """
  def find_records(fqdn, rtype \\ nil) when is_binary(fqdn) do
    with {:ok, mgr} <- get_manager(),
         json when is_binary(json) <- Native.manager_find_records(mgr, fqdn, rtype) do
      decode_records(json)
    else
      _ -> {:error, :failed_to_find_records}
    end
  end

  defp decode_records(json) do
    case Jason.decode(json) do
      {:ok, records} when is_list(records) -> {:ok, normalize_records(records)}
      _ -> {:error, :invalid_records_response}
    end
  end

  defp normalize_records(records) do
    Enum.map(records, fn rec ->
      %{
        name: Map.get(rec, "name", ""),
        rtype: Map.get(rec, "rtype", ""),
        ttl: Map.get(rec, "ttl", 300),
        data: Map.get(rec, "data", "")
      }
    end)
  end

  @doc """
  Add a DNS record to a zone.

  Parameters:
    - zone_name: Name of the zone (e.g., "example.com")
    - record_name: Name of the record (e.g., "www" or "www.example.com")
    - record_type: DNS record type (e.g., "A", "AAAA", "CNAME", "MX", "TXT")
    - ttl: Time to live in seconds (default 300)
    - data: Record data (e.g., IP address, hostname, text)

  Returns `{:ok, record}` on success, `{:error, reason}` on failure.
  """
  def add_record(zone_name, record_name, record_type, ttl, data)
      when is_binary(zone_name) and is_binary(record_name) and is_binary(record_type) and
             is_integer(ttl) and ttl > 0 and is_binary(data) do
    with {:ok, mgr} <- get_manager() do
      case Native.manager_add_record(mgr, zone_name, record_name, record_type, ttl, data) do
        true ->
<<<<<<< HEAD
          persist_record(zone_name, record_name, record_type, ttl, data)
=======
          # persist record to DB
          zone = Repo.get_by(Zone, name: zone_name)

          if zone do
            %Record{}
            |> Record.changeset(%{
              zone_id: zone.id,
              name: record_name,
              rtype: record_type,
              ttl: ttl,
              data: data
            })
            |> Repo.insert(on_conflict: :nothing)
          end
>>>>>>> 0131628384990b0e3356b3ae6e8309236db5a2e1

          {:ok, %{name: record_name, rtype: record_type, ttl: ttl, data: data}}

        false ->
          {:error, :failed_to_add_record}
      end
    end
  end

  def add_record(_, _, _, _, _), do: {:error, :invalid_record_params}

  @doc """
  Remove a DNS record from a zone.

  Parameters:
    - zone_name: Name of the zone
    - record_name: Name of the record
    - record_type: DNS record type

  Returns `{:ok, :deleted}` on success, `{:error, reason}` on failure.
  """
  def remove_record(zone_name, record_name, record_type)
      when is_binary(zone_name) and is_binary(record_name) and is_binary(record_type) do
    with {:ok, mgr} <- get_manager() do
      case Native.manager_remove_record(mgr, zone_name, record_name, record_type) do
        true ->
<<<<<<< HEAD
          delete_persisted_record(zone_name, record_name, record_type)
=======
          # delete from DB
          if zone = Repo.get_by(Zone, name: zone_name) do
            from(r in Record,
              where: r.zone_id == ^zone.id and r.name == ^record_name and r.rtype == ^record_type
            )
            |> Repo.delete_all()
          end
>>>>>>> 0131628384990b0e3356b3ae6e8309236db5a2e1

          {:ok, :deleted}

        false ->
          {:error, :record_not_found}
      end
    end
  end

  def remove_record(_, _, _), do: {:error, :invalid_record_params}

  @doc """
  Handle a raw DNS query (for testing/debugging).

  Returns the raw DNS response binary or nil if query cannot be handled.
  """
  def handle_query(query_binary) when is_binary(query_binary) do
    with {:ok, mgr} <- get_manager() do
      case Native.engine_handle_query(mgr, query_binary) do
        resp when is_binary(resp) -> {:ok, resp}
        nil -> {:error, :query_not_handled}
      end
    end
  end

  def handle_query(_), do: {:error, :invalid_query}

  defp persist_record(zone_name, record_name, record_type, ttl, data) do
    with %Zone{} = zone <- Repo.get_by(Zone, name: zone_name) do
      %Record{}
      |> Record.changeset(%{
        zone_id: zone.id,
        name: record_name,
        rtype: record_type,
        ttl: ttl,
        data: data
      })
      |> Repo.insert(on_conflict: :nothing)
    end
  end

  defp delete_persisted_record(zone_name, record_name, record_type) do
    with %Zone{} = zone <- Repo.get_by(Zone, name: zone_name) do
      from(r in Record,
        where: r.zone_id == ^zone.id and r.name == ^record_name and r.rtype == ^record_type
      )
      |> Repo.delete_all()
    end
  end
end
