defmodule HackflareWeb.DashController do
  @moduledoc """
  Controller for authenticated dashboard views.
  """
  use HackflareWeb, :controller
  alias Hackflare.Accounts
  alias Hackflare.DNS
  alias Hackflare.DNS.Zone
  alias Hackflare.Repo
  alias Hackflare.Support
  import Ecto.Query, only: [from: 2]

  def home(conn, _params) do
    render(conn, :dashboard, current_view: :home, current_user: get_current_user!(conn))
  end

  def domains(conn, _params) do
    current_user = get_current_user!(conn)
    zones = user_zones(current_user)

    zone_records =
      Enum.into(zones, %{}, fn zone ->
        {zone.name, zone.records}
      end)

    render(conn, :dashboard,
      current_view: :domains,
      zones: zones,
      zone_records: zone_records,
      current_user: get_current_user!(conn)
    )
  end

  def create_zone(conn, %{"zone_name" => zone_name, "zone_type" => zone_type})
      when is_binary(zone_name) and is_binary(zone_type) do
    current_user = get_current_user!(conn)

    case DNS.create_zone(String.trim(zone_name), String.trim(zone_type), current_user) do
      {:ok, _} ->
        conn
        |> put_flash(
          :info,
          "Zone #{zone_name} created. Verify nameservers before adding records."
        )
        |> redirect(to: ~p"/dash/domains")
    end
  end

  def create_zone(conn, _params) do
    conn
    |> put_flash(:error, "Invalid zone name.")
    |> redirect(to: ~p"/dash/domains")
  end

  def create_record(conn, params) do
    current_user = get_current_user!(conn)
    zone_name = String.trim(Map.get(params, "zone_name", ""))
    record_name = String.trim(Map.get(params, "record_name", ""))
    record_type = String.trim(Map.get(params, "record_type", ""))
    record_data = String.trim(Map.get(params, "record_data", ""))
    ttl = parse_ttl(Map.get(params, "ttl", "300"))

    with true <- zone_name != "",
         true <- record_type != "",
         true <- record_data != "",
         true <- is_integer(ttl) and ttl > 0,
          {:ok, _} <- DNS.add_record(zone_name, record_name, record_type, ttl, record_data, current_user) do
      conn
      |> put_flash(:info, "Record added to #{zone_name}.")
      |> redirect(to: ~p"/dash/domains")
    else
      _ ->
        conn
        |> put_flash(:error, "Failed to add DNS record.")
        |> redirect(to: ~p"/dash/domains")
    end
  end

  def reverify_zone(conn, %{"zone_name" => zone_name}) when is_binary(zone_name) do
    current_user = get_current_user!(conn)

    case DNS.verify_zone_nameservers(String.trim(zone_name), current_user) do
      {:ok, :verified} ->
        conn
        |> put_flash(:info, "Nameservers for #{zone_name} verified.")
        |> redirect(to: ~p"/dash/domains")

      {:error, :zone_not_found} ->
        conn
        |> put_flash(:error, "Zone #{zone_name} not found.")
        |> redirect(to: ~p"/dash/domains")

      {:error, :nameservers_mismatch} ->
        conn
        |> put_flash(:error, "Nameservers for #{zone_name} must be under *.ns.kirze.de.")
        |> redirect(to: ~p"/dash/domains")

      {:error, _reason} ->
        conn
        |> put_flash(:error, "Failed to verify nameservers for #{zone_name}.")
        |> redirect(to: ~p"/dash/domains")
    end
  end

  def reverify_zone(conn, _params) do
    conn
    |> put_flash(:error, "Invalid zone name.")
    |> redirect(to: ~p"/dash/domains")
  end

  def delete_zone(conn, %{"zone_name" => zone_name}) when is_binary(zone_name) do
    zone_name_decoded = String.replace(zone_name, "-", ".")
    current_user = get_current_user!(conn)

    case DNS.delete_zone(zone_name_decoded, current_user) do
      {:ok, _} ->
        conn
        |> put_flash(:info, "Zone #{zone_name_decoded} deleted successfully.")
        |> redirect(to: ~p"/dash/domains")

      {:error, :zone_not_found} ->
        conn
        |> put_flash(:error, "Zone #{zone_name_decoded} not found.")
        |> redirect(to: ~p"/dash/domains")

      {:error, _reason} ->
        conn
        |> put_flash(:error, "Failed to delete zone #{zone_name_decoded}.")
        |> redirect(to: ~p"/dash/domains")
    end
  end

  def delete_zone(conn, _params) do
    conn
    |> put_flash(:error, "Invalid zone name.")
    |> redirect(to: ~p"/dash/domains")
  end

  def settings(conn, _params) do
    render(conn, :dashboard, current_view: :settings, current_user: get_current_user!(conn))
  end

  def analytics(conn, _params) do
    render(conn, :dashboard, current_view: :analytics, current_user: get_current_user!(conn))
  end

  def notifications(conn, _params) do
    render(conn, :dashboard, current_view: :notifications, current_user: get_current_user!(conn))
  end

  def help(conn, _params) do
    render(conn, :dashboard,
      current_view: :help,
      form_message: "",
      current_user: get_current_user!(conn)
    )
  end

  def submit_help(conn, %{"help" => %{"message" => message}}) do
    with {:ok, user} <- get_current_user(conn),
         :ok <- Support.submit_help_request(user, message) do
      conn
      |> put_flash(:info, "Help message sent to Hackflare Slack.")
      |> redirect(to: ~p"/dash/help")
    else
      {:error, :not_authenticated} ->
        conn
        |> put_flash(:error, "You need to be signed in.")
        |> redirect(to: ~p"/")

      {:error, :empty_message} ->
        conn
        |> put_flash(:error, "Message cannot be empty.")
        |> render(:dashboard,
          current_view: :help,
          form_message: message,
          current_user: get_current_user!(conn)
        )

      {:error, :webhook_not_configured} ->
        conn
        |> put_flash(:error, "Support channel is not configured right now.")
        |> render(:dashboard,
          current_view: :help,
          form_message: message,
          current_user: get_current_user!(conn)
        )

      {:error, {:request_failed, _reason}} ->
        conn
        |> put_flash(:error, "Failed to send message to Slack.")
        |> render(:dashboard,
          current_view: :help,
          form_message: message,
          current_user: get_current_user!(conn)
        )

      {:error, {:webhook_rejected, _status, _body}} ->
        conn
        |> put_flash(:error, "Slack webhook rejected the request.")
        |> render(:dashboard,
          current_view: :help,
          form_message: message,
          current_user: get_current_user!(conn)
        )
    end
  end

  def submit_help(conn, _params) do
    conn
    |> put_flash(:error, "Invalid help request payload.")
    |> render(:dashboard,
      current_view: :help,
      form_message: "",
      current_user: get_current_user!(conn)
    )
  end

  defp get_current_user(conn) do
    case get_session(conn, :user_id) do
      nil ->
        {:error, :not_authenticated}

      user_id ->
        case Accounts.get_user(user_id) do
          nil -> {:error, :not_authenticated}
          user -> {:ok, user}
        end
    end
  end

  defp get_current_user!(conn) do
    case get_session(conn, :user_id) do
      nil -> nil
      user_id -> Accounts.get_user(user_id)
    end
  end

  defp parse_ttl(value) when is_binary(value) do
    case Integer.parse(value) do
      {ttl, ""} when ttl > 0 -> ttl
      _ -> nil
    end
  end

  defp parse_ttl(value) when is_integer(value) and value > 0, do: value
  defp parse_ttl(_), do: nil

  defp user_zones(%{is_admin: true}) do
    Repo.all(Zone) |> Repo.preload(:records)
  end

  defp user_zones(%{id: user_id}) do
    from(z in Zone, where: z.user_id == ^user_id)
    |> Repo.all()
    |> Repo.preload(:records)
  end

  defp user_zones(_current_user), do: []
end
