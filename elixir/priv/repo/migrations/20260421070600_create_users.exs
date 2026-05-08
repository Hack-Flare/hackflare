defmodule Hackflare.Repo.Migrations.CreateUsers do
  use Ecto.Migration

  def change do
    create table(:users) do
      add :slack_id, :string
      add :email, :string
      add :name, :string
      add :verification_status, :string
      add :ysws_eligible, :boolean, default: false, null: false

      timestamps(type: :utc_datetime)
    end

    create unique_index(:users, [:email])
    create unique_index(:users, [:slack_id])
  end
end
