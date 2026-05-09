defmodule Hackflare.Support do
  @moduledoc """
  Support helpers for sending user help requests to Slack.
  """

  alias Hackflare.Accounts.User

  @spec submit_help_request(User.t(), binary()) ::
          :ok
          | {:error,
             :empty_message
             | :webhook_not_configured
             | {:request_failed, term()}
             | {:webhook_rejected, non_neg_integer(), term()}}
  def submit_help_request(%User{} = user, message) when is_binary(message) do
    trimmed_message = String.trim(message)
    webhook_url = Hackflare.Settings.slack_help_webhook_url()

    cond do
      trimmed_message == "" ->
        {:error, :empty_message}

      webhook_url == "" ->
        {:error, :webhook_not_configured}

      true ->
        do_submit_help_request(webhook_url, user, trimmed_message)
    end
  end

  defp do_submit_help_request(webhook_url, user, message) do
    payload = %{
      text: """
      :neocat_floof_explode: *Hackflare Help Request*
      Slack ID: <@#{user.slack_id}>
      Name: #{user.name}
      Email: #{user.email}
      Message:
      #{message}
      """
    }

    case Req.post(webhook_url, json: payload) do
      {:ok, %Req.Response{status: status}} when status in 200..299 ->
        :ok

      {:ok, %Req.Response{status: status, body: body}} ->
        {:error, {:webhook_rejected, status, body}}

      {:error, reason} ->
        {:error, {:request_failed, reason}}
    end
  end
end
