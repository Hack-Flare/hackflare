defmodule Hackflare.Accounts do
  @moduledoc """
  Account-related helpers for creating and fetching users.

  This module wraps common user operations used by the web controllers.
  """

  alias Hackflare.Accounts.User
  alias Hackflare.Repo

  def admin_emails do
    Hackflare.Settings.admin_emails()
  end

  def admin_user?(%User{is_admin: true}), do: true

  def admin_user?(%User{email: email}) when is_binary(email) do
    email in admin_emails()
  end

  def admin_user?(_user), do: false

  def register_or_signin(profile) do
    user_params = %{
      slack_id: profile["slack_id"],
      email: profile["email"],
      name: profile["name"],
      verification_status: profile["verification_status"],
      ysws_eligible: profile["ysws_eligible"] || false,
      is_admin: profile["email"] in admin_emails()
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

  def list_users do
    Repo.all(User)
  end

  def get_user(id), do: Repo.get(User, id)
end
