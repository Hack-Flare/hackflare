defmodule Hackflare.Repo do
  @moduledoc """
  The Ecto Repository for Hackflare database operations.

  This module provides the database connection and query interface for all
  persistent data in Hackflare. It uses PostgreSQL as the database backend
  and handles connection pooling, migrations, and Ecto query execution.

  ## Usage

      # Query examples
      Hackflare.Repo.all(MySchema)
      Hackflare.Repo.get(MySchema, id)
      Hackflare.Repo.insert(changeset)
      Hackflare.Repo.update(changeset)
      Hackflare.Repo.delete(changesetOrStruct)

  ## Configuration

  Database configuration is loaded from `config/runtime.exs` and environment variables:
  - `DATABASE_URL` - Full PostgreSQL connection string (typically used in production)
  - Or: `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASSWORD`

  ## Migrations

  Run migrations with:

      mix ecto.migrate

  Create new migrations with:

      mix ecto.gen.migration migration_name
  """
  use Ecto.Repo,
    otp_app: :hackflare,
    adapter: Ecto.Adapters.Postgres
end
