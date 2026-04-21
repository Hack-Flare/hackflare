defmodule Hackflare.HackClubAuth do

  def config do
    Application.get_env(:hackflare, :auth)
  end

  def get_authorize_url(redirect_uri) do
    config()
    |> Keyword.put(:redirect_uri, redirect_uri)
    |> Assent.Strategy.OIDC.authorize_url()
  end

  def callback(params, redirect_uri, session_params \\ %{}) do
    config()
    |> Keyword.put(:redirect_uri, redirect_uri)
    |> Keyword.put(:session_params, session_params)
    |> Assent.Strategy.OIDC.callback(params)
  end
end
