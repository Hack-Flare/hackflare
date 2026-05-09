defmodule Hackflare.Repo.Migrations.AddUserIdToDnsZones do
  use Ecto.Migration

  def change do
    alter table(:dns_zones) do
      add :user_id, references(:users, on_delete: :delete_all)
    end

    create index(:dns_zones, [:user_id])
  end
end
