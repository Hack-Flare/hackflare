defmodule Hackflare.HackClubAuth do
  @moduledoc """
  Small helper around the Hack Club OIDC configuration and Assent calls.

  Provides helpers to build the authorization URL and handle the
  OIDC callback using the `Assent.Strategy.OIDC` strategy.
  """

  alias Assent.Strategy.OIDC

  def config do
    Hackflare.Settings.auth_config()
  end

  def get_authorize_url(redirect_uri) do
    config()
    |> Keyword.put(:redirect_uri, redirect_uri)
    |> OIDC.authorize_url()
  end

  def callback(params, redirect_uri, session_params \\ %{}) do
    config()
    |> Keyword.put(:redirect_uri, redirect_uri)
    |> Keyword.put(:session_params, session_params)
    |> OIDC.callback(params)
  end
end
