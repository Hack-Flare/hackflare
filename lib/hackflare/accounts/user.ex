defmodule Hackflare.Accounts.User do
  @moduledoc """
  Ecto schema for application users and related changeset helpers.
  """
  use Ecto.Schema
  import Ecto.Changeset

  schema "users" do
    field(:slack_id, :string)
    field(:email, :string)
    field(:name, :string)
    field(:verification_status, :string)
    field(:ysws_eligible, :boolean, default: false)
    field(:is_admin, :boolean, default: false)

    timestamps(type: :utc_datetime)
  end

  @type t :: %__MODULE__{}

  @doc false
  def changeset(user, attrs) do
    user
    |> cast(attrs, [:slack_id, :email, :name, :verification_status, :ysws_eligible, :is_admin])
    |> validate_required([:slack_id, :email, :name, :verification_status, :ysws_eligible, :is_admin])
    |> unique_constraint(:slack_id)
    |> unique_constraint(:email)
  end
end
