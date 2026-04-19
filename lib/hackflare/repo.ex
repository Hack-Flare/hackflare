defmodule Hackflare.Repo do
  use Ecto.Repo,
    otp_app: :hackflare,
    adapter: Ecto.Adapters.Postgres
end
