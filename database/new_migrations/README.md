# Pending Migrations

Place migrations required by `hackflare-app` in this directory while the merged application is being verified.

Do not run or add these migrations to the active database migration set yet.

Once `hackflare-app` is ready and the legacy API/frontend crates are removed, move the migration files into `database/migrations/` before deploying.

Keep migration filenames ordered with the existing migration timestamps and include both `.up.sql` and `.down.sql` files when rollback support is appropriate.
