defmodule Hackflare.Accounts do
  alias Hackflare.Repo
  alias Hackflare.Accounts.User

  def register_or_signin(profile) do
    user_params = %{
      slack_id: profile["slack_id"],
      email: profile["email"],
      name: profile["name"],
      verification_status: profile["verification_status"],
      ysws_eligible: profile["ysws_eligible"] || false
    }

    case Repo.get_by(User, slack_id: user_params.slack_id) do
      nil ->
        %User{}
        |> User.changeset(user_params)
        |> Repo.insert()

      user ->
        user
        |> User.changeset(user_params)
        |> Repo.update()
    end
  end

  def get_user(id), do: Repo.get(User, id)
end
