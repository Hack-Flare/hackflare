defmodule HackflareWeb.AuthController do
  use HackflareWeb, :controller
  alias Hackflare.Accounts
  alias Hackflare.HackClubAuth

  @doc """
  Initiates the Hack Club OAuth/OIDC flow.
  """
  def request(conn, _params) do
    # Ensure this matches the Redirect URI in your Hack Club Dev Dashboard
    redirect_uri = url(~p"/auth/callback")

    case HackClubAuth.get_authorize_url(redirect_uri) do
      {:ok, %{url: url, session_params: session_params}} ->
        conn
        # Store full session_params (state, nonce, code_verifier, etc.)
        |> put_session(:oauth_session_params, session_params)
        |> put_session(:oauth_state, session_params[:state])
        |> put_session(:oauth_nonce, session_params[:nonce])
        |> redirect(external: url)

      {:error, _reason} ->
        conn
        |> put_flash(:error, "Could not contact Hack Club Auth.")
        |> redirect(to: ~p"/")
    end
  end

  @doc """
  Handles the callback from Hack Club.
  """
  def callback(conn, params) do
    redirect_uri = url(~p"/auth/callback")

    # Retrieve the stored session params (state, nonce, code_verifier)
    session_params = get_session(conn, :oauth_session_params, %{})

    # Merge state/nonce into callback params for convenience
    callback_params =
      params
      |> Map.put("state", session_params[:state])
      |> Map.put("nonce", session_params[:nonce])

    case HackClubAuth.callback(callback_params, redirect_uri, session_params) do
      {:ok, %{user: profile}} ->
        case Accounts.register_or_signin(profile) do
          {:ok, user} ->
            conn
            |> put_flash(:info, "Welcome to HackFlare, #{user.name}!")
            |> put_session(:user_id, user.id)
            # Good practice to rotate session on login
            |> configure_session(renew: true)
            |> redirect(to: ~p"/")

          {:error, _changeset} ->
            conn
            |> put_flash(:error, "Failed to save user data.")
            |> redirect(to: ~p"/")
        end

      {:error, _reason} ->
        conn
        |> put_flash(:error, "Hack Club authentication failed.")
        |> redirect(to: ~p"/")
    end
  end

  @doc """
  Logs the user out by clearing the session.
  """
  def logout(conn, _params) do
    conn
    |> configure_session(drop: true)
    |> put_flash(:info, "Logged out successfully.")
    |> redirect(to: ~p"/")
  end
end
