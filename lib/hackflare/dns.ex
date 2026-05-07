defmodule Hackflare.DnsQueryMetric do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :integer, autogenerate: false}
  schema "dns_query_metrics" do
    field(:udp_count, :integer, default: 0)
    field(:tcp_count, :integer, default: 0)

    timestamps()
  end

  def changeset(metric, attrs) do
    metric
    |> cast(attrs, [:udp_count, :tcp_count])
    |> validate_required([:udp_count, :tcp_count])
  end
end

defmodule Hackflare.DNS.Zone do
  use Ecto.Schema
  import Ecto.Changeset

  schema "dns_zones" do
    belongs_to(:user, Hackflare.Accounts.User)
    field(:name, :string)
    field(:type, :string, default: "root")
    field(:ns_verified, :boolean, default: false)
    has_many(:records, Hackflare.DNS.Record, foreign_key: :zone_id)

    timestamps()
  end

  def changeset(zone, attrs) do
    zone
    |> cast(attrs, [:user_id, :name, :type, :ns_verified])
    |> validate_required([:user_id, :name])
    |> unique_constraint(:name)
  end
end

defmodule Hackflare.DNS.Record do
  use Ecto.Schema
  import Ecto.Changeset

  schema "dns_records" do
    field(:name, :string)
    field(:rtype, :string)
    field(:ttl, :integer)
    field(:data, :string)
    belongs_to(:zone, Hackflare.DNS.Zone, foreign_key: :zone_id)

    timestamps()
  end

  def changeset(record, attrs) do
    record
    |> cast(attrs, [:zone_id, :name, :rtype, :ttl, :data])
    |> validate_required([:zone_id, :name, :rtype, :ttl, :data])
  end
end

defmodule Hackflare.DNS do
  @moduledoc """
  DNS zone and record management context.

  Provides a high-level interface for managing DNS zones and records,
  wrapping the low-level Rust NIF calls from Hackflare.Native.
  """

  alias Hackflare.Accounts
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
  Zones must be owned by a signed-in user.
  """
  def create_zone(zone_name) when is_binary(zone_name) do
    {:error, :owner_required}
  end

  def create_zone(_), do: {:error, :invalid_zone_name}

  def create_zone(zone_name, zone_type) when is_binary(zone_name) and is_binary(zone_type) do
    {:error, :owner_required}
  end

  def create_zone(zone_name, zone_type, current_user)
      when is_binary(zone_name) and is_binary(zone_type) do
    if is_nil(current_user), do: {:error, :owner_required}, else: insert_zone(zone_name, zone_type, current_user)
  end

  defp insert_zone(zone_name, zone_type, current_user) do
    zone_attrs =
      %{name: zone_name, type: zone_type}
      |> maybe_put_user_id(current_user)

    %Zone{}
    |> Zone.changeset(zone_attrs)
    |> Repo.insert()
    |> handle_zone_insert(zone_name)
  end

  defp handle_zone_insert({:ok, _zone}, zone_name), do: {:ok, zone_name}

  defp handle_zone_insert({:error, %Ecto.Changeset{} = changeset}, _zone_name) do
    if zone_name_conflict?(changeset), do: {:error, :zone_already_exists}, else: {:error, changeset}
  end

  defp handle_zone_insert({:error, reason}, _zone_name), do: {:error, reason}

  @doc """
  Delete a DNS zone.

  Returns `{:ok, zone_name}` on success, `{:error, reason}` on failure.
  """
  def delete_zone(zone_name) when is_binary(zone_name) do
    {:error, :owner_required}
  end

  def delete_zone(_), do: {:error, :invalid_zone_name}

  def delete_zone(zone_name, current_user) when is_binary(zone_name) do
    case get_zone(zone_name, current_user) do
      nil ->
        {:error, :zone_not_found}

      %Zone{ns_verified: true} = zone ->
        delete_verified_zone(zone)

      %Zone{ns_verified: false} = zone ->
        delete_zone_from_db(zone)
        {:ok, zone_name}
    end
  end

  defp delete_verified_zone(%Zone{name: zone_name} = zone) do
    with {:ok, mgr} <- get_manager(),
         true <- Native.manager_delete_zone(mgr, zone_name) do
      delete_zone_from_db(zone)
      {:ok, zone_name}
    else
      false -> {:error, :failed_to_delete_zone}
      {:error, _reason} = error -> error
    end
  end

  defp delete_zone_from_db(%Zone{id: zone_id}) do
    from(z in Zone, where: z.id == ^zone_id) |> Repo.delete_all()
  end

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
    {:error, :owner_required}
  end

  def add_record(_, _, _, _, _), do: {:error, :invalid_record_params}

  def add_record(zone_name, record_name, record_type, ttl, data, current_user)
      when is_binary(zone_name) and is_binary(record_name) and is_binary(record_type) and
             is_integer(ttl) and ttl > 0 and is_binary(data) do
    with {:ok, zone} <- ensure_zone_verified(zone_name, current_user),
         {:ok, mgr} <- get_manager(),
         true <- Native.manager_add_record(mgr, zone_name, record_name, record_type, ttl, data) do
      persist_record(zone, record_name, record_type, ttl, data)

      {:ok, %{name: record_name, rtype: record_type, ttl: ttl, data: data}}
    else
      false -> {:error, :failed_to_add_record}
      {:error, _reason} = error -> error
    end
  end

  def add_record(_, _, _, _, _, _), do: {:error, :invalid_record_params}

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
    {:error, :owner_required}
  end

  def remove_record(_, _, _), do: {:error, :invalid_record_params}

  def remove_record(zone_name, record_name, record_type, current_user)
      when is_binary(zone_name) and is_binary(record_name) and is_binary(record_type) do
    with {:ok, zone} <- ensure_zone_verified(zone_name, current_user),
         {:ok, mgr} <- get_manager(),
         true <- Native.manager_remove_record(mgr, zone_name, record_name, record_type) do
      delete_persisted_records(zone, record_name, record_type)
      {:ok, :deleted}
    else
      false -> {:error, :record_not_found}
      {:error, _reason} = error -> error
    end
  end

  def remove_record(_, _, _, _), do: {:error, :invalid_record_params}

  @doc """
  Update an existing DNS record in a verified zone.

  The old record is identified by name and type. On success, the record is
  replaced with the new values in both the DNS manager and database.
  """
  def update_record(
        zone_name,
        old_record_name,
        old_record_type,
        new_record_name,
        new_record_type,
        ttl,
        data,
        current_user
      )
      when is_binary(zone_name) and is_binary(old_record_name) and is_binary(old_record_type) and
             is_binary(new_record_name) and is_binary(new_record_type) and is_integer(ttl) and
             ttl > 0 and is_binary(data) do
    persisted_old_record = get_persisted_record(zone_name, old_record_name, old_record_type)

    with {:ok, zone} <- ensure_zone_verified(zone_name, current_user),
         {:ok, mgr} <- get_manager(),
         true <- Native.manager_remove_record(mgr, zone_name, old_record_name, old_record_type) do
      apply_record_update(
        mgr,
        zone,
        zone_name,
        old_record_name,
        old_record_type,
        new_record_name,
        new_record_type,
        ttl,
        data,
        persisted_old_record
      )
    else
      false -> {:error, :record_not_found}
      {:error, _reason} = error -> error
    end
  end

  def update_record(_, _, _, _, _, _, _, _), do: {:error, :invalid_record_params}

  defp apply_record_update(
         mgr,
         zone,
         zone_name,
         old_record_name,
         old_record_type,
         new_record_name,
         new_record_type,
         ttl,
         data,
         persisted_old_record
       ) do
    case Native.manager_add_record(mgr, zone_name, new_record_name, new_record_type, ttl, data) do
      true ->
        delete_persisted_records(zone, old_record_name, old_record_type)
        persist_record(zone, new_record_name, new_record_type, ttl, data)

        {:ok,
         %{name: new_record_name, rtype: new_record_type, ttl: ttl, data: data, zone: zone_name}}

      false ->
        restore_original_record(mgr, zone_name, old_record_name, old_record_type, persisted_old_record)
        {:error, :failed_to_update_record}
    end
  end

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

  defp persist_record(%Zone{id: zone_id}, record_name, record_type, ttl, data) do
    %Record{}
    |> Record.changeset(%{
      zone_id: zone_id,
      name: record_name,
      rtype: record_type,
      ttl: ttl,
      data: data
    })
    |> Repo.insert(on_conflict: :nothing)
  end

  defp delete_persisted_records(%Zone{id: zone_id}, record_name, record_type) do
    from(r in Record,
      where: r.zone_id == ^zone_id and r.name == ^record_name and r.rtype == ^record_type
    )
    |> Repo.delete_all()
  end

  defp get_persisted_record(zone_name, record_name, record_type) do
    from(r in Record,
      join: z in Zone,
      on: r.zone_id == z.id,
      where: z.name == ^zone_name and r.name == ^record_name and r.rtype == ^record_type,
      limit: 1
    )
    |> Repo.one()
  end

  defp restore_original_record(mgr, zone_name, record_name, record_type, %Record{} = record) do
    _ =
      Native.manager_add_record(mgr, zone_name, record_name, record_type, record.ttl, record.data)

    :ok
  end

  defp restore_original_record(_mgr, _zone_name, _record_name, _record_type, _record), do: :ok

  defp ensure_zone_verified(zone_name, current_user) do
    case get_zone(zone_name, current_user) do
      %Zone{ns_verified: true} = zone -> {:ok, zone}
      %Zone{} -> {:error, :ns_not_verified}
      nil -> {:error, :zone_not_found}
    end
  end

  @doc """
  Verify that a zone's NS delegation uses Hackflare nameservers.

  A delegation is considered valid when all delegated NS hostnames match the
  pattern `<label>.ns.kirze.de` (case-insensitive).

  If verification succeeds the zone will be marked `ns_verified: true` and the
  zone + its records will be created/loaded into the running DNS manager.

  Returns `{:ok, :verified}` on success or `{:error, reason}` on failure.
  """
  def verify_zone_nameservers(zone_name) when is_binary(zone_name) do
    {:error, :owner_required}
  end

  def verify_zone_nameservers(zone_name, current_user) when is_binary(zone_name) do
    case get_zone(zone_name, current_user) do
      nil ->
        {:error, :zone_not_found}

      %Zone{} = zone ->
        # Query NS records using the Erlang resolver
        try do
          ns_raw =
            :inet_res.lookup(
              String.to_charlist(zone_name),
              :in,
              :ns,
              resolver_opts(),
              2_000
            )

          ns_list =
            ns_raw
            |> Enum.map(&List.to_string/1)
            |> Enum.map(&String.trim_trailing(&1, "."))
            |> Enum.map(&String.downcase/1)

          matched =
            ns_list != [] and
              Enum.all?(ns_list, fn ns ->
                String.match?(ns, ~r/^[a-z0-9-]+\.ns\.kirze\.de$/)
              end)

          if matched do
            # mark as verified and create zone in manager + add persisted records
            zone
            |> Ecto.Changeset.change(ns_verified: true)
            |> Repo.update()

            with {:ok, mgr} <- get_manager() do
              _ = Native.manager_create_zone(mgr, zone_name)
              zone = Repo.preload(zone, :records)

              Enum.each(zone.records, fn rec ->
                _ =
                  Native.manager_add_record(
                    mgr,
                    zone.name,
                    rec.name,
                    rec.rtype,
                    rec.ttl,
                    rec.data
                  )
              end)
            end

            {:ok, :verified}
          else
            {:error, :nameservers_mismatch}
          end
        rescue
          _ -> {:error, :dns_lookup_failed}
        end
    end
  end

  defp maybe_put_user_id(attrs, %Accounts.User{id: user_id}) when is_integer(user_id) do
    Map.put(attrs, :user_id, user_id)
  end

  defp maybe_put_user_id(attrs, _current_user), do: attrs

  defp resolver_opts do
    [
      nameservers: [{{1, 1, 1, 1}, 53}, {{8, 8, 8, 8}, 53}]
    ]
  end

  defp zone_name_conflict?(%Ecto.Changeset{errors: errors}) do
    Enum.any?(errors, fn
      {:name, {_message, opts}} -> opts[:constraint] == :unique
      _ -> false
    end)
  end

  defp get_zone(zone_name, nil) do
    Repo.get_by(Zone, name: zone_name)
  end

  defp get_zone(zone_name, %Accounts.User{} = current_user) do
    query =
      if Accounts.admin_user?(current_user) do
        from(z in Zone, where: z.name == ^zone_name)
      else
        from(z in Zone, where: z.name == ^zone_name and z.user_id == ^current_user.id)
      end

    Repo.one(query)
  end
end

defmodule Hackflare.DNS.Record do
  use Ecto.Schema
  import Ecto.Changeset

  schema "dns_records" do
    field(:name, :string)
    field(:rtype, :string)
    field(:ttl, :integer)
    field(:data, :string)
    belongs_to(:zone, Hackflare.DNS.Zone, foreign_key: :zone_id)

    timestamps()
  end

  def changeset(record, attrs) do
    record
    |> cast(attrs, [:zone_id, :name, :rtype, :ttl, :data])
    |> validate_required([:zone_id, :name, :rtype, :ttl, :data])
  end
end

defmodule Hackflare.DNS.Zone do
  use Ecto.Schema
  import Ecto.Changeset

  schema "dns_zones" do
    belongs_to(:user, Hackflare.Accounts.User)
    field(:name, :string)
    field(:type, :string, default: "root")
    field(:ns_verified, :boolean, default: false)
    has_many(:records, Hackflare.DNS.Record, foreign_key: :zone_id)

    timestamps()
  end

  def changeset(zone, attrs) do
    zone
    |> cast(attrs, [:user_id, :name, :type, :ns_verified])
    |> validate_required([:user_id, :name])
    |> unique_constraint(:name)
  end
end

defmodule Hackflare.DnsQueryMetric do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :integer, autogenerate: false}
  schema "dns_query_metrics" do
    field(:udp_count, :integer, default: 0)
    field(:tcp_count, :integer, default: 0)

    timestamps()
  end

  def changeset(metric, attrs) do
    metric
    |> cast(attrs, [:udp_count, :tcp_count])
    |> validate_required([:udp_count, :tcp_count])
  end
end
