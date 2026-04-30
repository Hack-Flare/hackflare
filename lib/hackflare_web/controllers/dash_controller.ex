defmodule HackflareWeb.DashController do
  @moduledoc """
  Controller for authenticated dashboard views.
  """
  use HackflareWeb, :controller
  alias Hackflare.Accounts
  alias Hackflare.Support

  def home(conn, _params) do
    render(conn, :dashboard, current_view: :home, current_user: get_current_user!(conn))
  end

  def domains(conn, _params) do
    render(conn, :dashboard, current_view: :domains, current_user: get_current_user!(conn))
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
end
