defmodule Hackflare.Repo.Migrations.CreateAppSettings do
  use Ecto.Migration

  def change do
    create table(:app_settings) do
      add :name, :string, null: false
      add :data, :map, null: false, default: %{}

      timestamps(type: :utc_datetime)
    end

    create unique_index(:app_settings, [:name])

    execute("""
    INSERT INTO app_settings (name, data, inserted_at, updated_at)
    VALUES ('runtime', '{}'::jsonb, now(), now())
    ON CONFLICT (name) DO NOTHING;
    """)
  end
end
