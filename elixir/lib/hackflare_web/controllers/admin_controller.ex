defmodule HackflareWeb.AdminController do
  @moduledoc """
  Controller for the admin panel.
  """

  use HackflareWeb, :controller

  alias Hackflare.Accounts
  alias Hackflare.DNS.Zone
  alias Hackflare.Repo
  alias Hackflare.Settings

  def index(conn, _params) do
    users = Accounts.list_users()
    settings = Settings.runtime()
    zones = Repo.all(Zone) |> Repo.preload([:records, :user])

    stats = %{
      total_users: length(users),
      total_domains: length(zones),
      admin_users: Enum.count(users, &Accounts.admin_user?/1),
      verified_users: Enum.count(users, &(&1.verification_status == "verified")),
      eligible_users: Enum.count(users, & &1.ysws_eligible)
    }

    render(conn, :index,
      current_view: :admin,
      users: users,
      zones: zones,
      stats: stats,
      metrics: Hackflare.Metrics.get_metrics(),
      settings: settings,
      current_user: get_current_user!(conn)
    )
  end

  def update(conn, %{"settings" => settings_params}) do
    case Settings.update_runtime(settings_params) do
      {:ok, _setting} ->
        _ = Settings.restart_nameserver()

        conn
        |> put_flash(:info, "Admin settings saved.")
        |> redirect(to: ~p"/admin")

      {:error, :settings_table_missing} ->
        conn
        |> put_flash(:error, "Database unavailable; settings table missing.")
        |> redirect(to: ~p"/admin")

      {:error, %Ecto.Changeset{} = cs} ->
        error_msg =
          case cs.errors do
            [{_field, {msg, _opts}} | _] -> String.capitalize(msg)
            _ -> "Unable to save admin settings."
          end

        conn
        |> put_flash(:error, error_msg)
        |> redirect(to: ~p"/admin")
    end
  end

  def update(conn, _params) do
    conn
    |> put_flash(:error, "Invalid settings payload.")
    |> redirect(to: ~p"/admin")
  end

  defp get_current_user!(conn) do
    case get_session(conn, :user_id) do
      nil -> nil
      user_id -> Accounts.get_user(user_id)
    end
  end
end
