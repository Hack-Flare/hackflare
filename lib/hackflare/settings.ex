defmodule Hackflare.Settings do
  @moduledoc """
  Runtime settings backed by the database with environment fallbacks.

  This lets the admin panel override app-level configuration without requiring
  a deploy, while still keeping bootstrap-only values in the environment.
  """

  import Ecto.Query

  alias Hackflare.Repo

  @runtime_name "runtime"

  defmodule AppSetting do
    @moduledoc false

    use Ecto.Schema
    import Ecto.Changeset

    schema "app_settings" do
      field(:name, :string)
      field(:data, :map, default: %{})

      timestamps(type: :utc_datetime)
    end

    def changeset(setting, attrs) do
      setting
      |> cast(attrs, [:name, :data])
      |> validate_required([:name, :data])
      |> unique_constraint(:name)
    end
  end

  def runtime do
    deep_merge(default_runtime(), runtime_overrides())
  end

  def admin_emails do
    runtime()
    |> Map.get(:admin_emails, "")
    |> String.split(",", trim: true)
    |> Enum.map(&String.trim/1)
    |> Enum.reject(&(&1 == ""))
  end

  def slack_help_webhook_url do
    runtime() |> Map.get(:slack_help_webhook_url, "")
  end

  def auth_config do
    auth = runtime() |> Map.get(:auth, %{})

    [
      client_id: Map.get(auth, :client_id, default_runtime().auth.client_id),
      client_secret: Map.get(auth, :client_secret, default_runtime().auth.client_secret),
      redirect_uri: Map.get(auth, :redirect_uri, default_runtime().auth.redirect_uri),
      base_url: Map.get(auth, :base_url, default_runtime().auth.base_url),
      openid_configuration_uri:
        Map.get(auth, :openid_configuration_uri, default_runtime().auth.openid_configuration_uri),
      authorization_params: [
        scope: Map.get(auth, :authorization_scope, default_runtime().auth.authorization_scope)
      ],
      strategy: Assent.Strategy.OIDC
    ]
  end

  def dns_config do
    dns = runtime() |> Map.get(:dns, %{})
    soa = Map.get(dns, :soa, %{})

    %{
      bind: Map.get(dns, :bind, default_runtime().dns.bind),
      port: parse_integer(Map.get(dns, :port, default_runtime().dns.port), 53),
      zones_file: Map.get(dns, :zones_file, default_runtime().dns.zones_file),
      soa: %{
        mname: Map.get(soa, :mname, default_runtime().dns.soa.mname),
        rname: Map.get(soa, :rname, default_runtime().dns.soa.rname),
        serial:
          parse_integer(Map.get(soa, :serial, default_runtime().dns.soa.serial), 2_026_042_000),
        refresh: parse_integer(Map.get(soa, :refresh, default_runtime().dns.soa.refresh), 1800),
        retry: parse_integer(Map.get(soa, :retry, default_runtime().dns.soa.retry), 900),
        expire: parse_integer(Map.get(soa, :expire, default_runtime().dns.soa.expire), 604_800),
        minimum: parse_integer(Map.get(soa, :minimum, default_runtime().dns.soa.minimum), 86_400),
        ttl: parse_integer(Map.get(soa, :ttl, default_runtime().dns.soa.ttl), 86_400)
      }
    }
  end

  def runtime_overrides do
    case Repo.one(from(setting in AppSetting, where: setting.name == ^@runtime_name, limit: 1)) do
      %AppSetting{data: data} when is_map(data) -> to_atom_map(data)
      _ -> %{}
    end
  rescue
    Ecto.QueryError -> %{}
    DBConnection.ConnectionError -> %{}
    Postgrex.Error -> %{}
  end

  def update_runtime(attrs) when is_map(attrs) do
    data = deep_merge(runtime_overrides(), to_atom_map(attrs))

    changeset =
      %AppSetting{}
      |> AppSetting.changeset(%{name: @runtime_name, data: data})

    now = DateTime.utc_now() |> DateTime.truncate(:second)

    Repo.insert(changeset,
      on_conflict: [set: [data: data, updated_at: now]],
      conflict_target: [:name]
    )
  rescue
    Ecto.QueryError -> {:error, :settings_table_missing}
    DBConnection.ConnectionError -> {:error, :settings_table_missing}
    Postgrex.Error -> {:error, :settings_table_missing}
  end

  def restart_nameserver do
    case Supervisor.restart_child(Hackflare.Supervisor, Hackflare.Nameserver) do
      {:ok, _pid} -> :ok
      {:ok, _old_pid, _new_pid} -> :ok
      {:error, reason} -> {:error, reason}
    end
  end

  defp default_runtime do
    %{
      admin_emails: get_admin_emails_default(),
      slack_help_webhook_url: System.get_env("SLACK_HELP_WEBHOOK_URL") || "",
      auth: default_auth(),
      dns: default_dns()
    }
  end

  defp get_admin_emails_default do
    case Application.get_env(:hackflare, :admin_emails, nil) do
      list when is_list(list) ->
        Enum.map_join(list, ", ", &if(is_atom(&1), do: to_string(&1), else: &1))

      _ ->
        System.get_env("HACKFLARE_ADMIN_EMAILS", "")
    end
  end

  defp default_auth do
    %{
      client_id: default_auth_client_id(),
      client_secret: default_auth_client_secret(),
      redirect_uri: default_auth_redirect_uri(),
      base_url: default_auth_base_url(),
      openid_configuration_uri: default_auth_openid_configuration_uri(),
      authorization_scope: "openid profile email slack_id verification_status"
    }
  end

  defp default_auth_client_id do
    System.get_env("HCA_CID") || System.get_env("hca_cid") || "fuhh-you-forgot-set-this"
  end

  defp default_auth_client_secret do
    System.get_env("HCA_SEC") || System.get_env("hca_sec") || "fuhh-you-forgot-set-this-too"
  end

  defp default_auth_redirect_uri do
    System.get_env("HCA_REDIR_URL") || System.get_env("hca_redir_url") ||
      "http://localhost:4000/auth/callback"
  end

  defp default_auth_base_url do
    System.get_env("HCA_BASE_URL") || System.get_env("hca_base_url") ||
      "https://auth.hackclub.com"
  end

  defp default_auth_openid_configuration_uri do
    System.get_env("HCA_OPENID_CONFIG_URI") || System.get_env("hca_openid_config_uri") ||
      "/.well-known/openid-configuration"
  end

  defp default_dns do
    %{
      bind: System.get_env("DNS_BIND") || "0.0.0.0",
      port: System.get_env("DNS_PORT", "53"),
      zones_file: System.get_env("DNS_ZONES_FILE") || "_build/dns_zones.json",
      soa: default_soa()
    }
  end

  defp default_soa do
    %{
      mname: System.get_env("DNS_SOA_MNAME") || "a.root-servers.net.",
      rname: System.get_env("DNS_SOA_RNAME") || "nstld.verisign-grs.com.",
      serial: System.get_env("DNS_SOA_SERIAL", "2026042000"),
      refresh: System.get_env("DNS_SOA_REFRESH", "1800"),
      retry: System.get_env("DNS_SOA_RETRY", "900"),
      expire: System.get_env("DNS_SOA_EXPIRE", "604800"),
      minimum: System.get_env("DNS_SOA_MINIMUM", "86400"),
      ttl: System.get_env("DNS_SOA_TTL", "86400")
    }
  end

  defp deep_merge(%{} = left, %{} = right) do
    Map.merge(left, right, fn _key, left_value, right_value ->
      case {left_value, right_value} do
        {%{} = left_map, %{} = right_map} -> deep_merge(left_map, right_map)
        {_, right} -> right
      end
    end)
  end

  defp to_atom_map(%{} = map) do
    Map.new(map, fn {key, value} -> {to_atom_key(key), normalize_value(value)} end)
  end

  defp normalize_value(%{} = map), do: to_atom_map(map)
  defp normalize_value(value), do: value

  defp to_atom_key(key) when is_atom(key), do: key
  defp to_atom_key(key) when is_binary(key), do: String.to_existing_atom(key)

  defp parse_integer(value, _default) when is_integer(value), do: value

  defp parse_integer(value, default) when is_binary(value) do
    case Integer.parse(String.trim(value)) do
      {integer, ""} -> integer
      _ -> default
    end
  end

  defp parse_integer(_value, default), do: default
end
