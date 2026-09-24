default: check

# Format all Rust crates.
fmt:
	cargo fmt --all

# Verify Rust formatting without changing files.
fmt-check:
	cargo fmt --all --check

# Build the merged application.
build:
	cargo build -p hackflare-app

alias build-app := build

# Build every workspace crate.
build-workspace:
	cargo build --workspace

# Run the merged application's tests. Tests start an embedded PostgreSQL when
# DATABASE_URL is not set, so Docker and a system PostgreSQL are optional.
test-app:
	cargo test -p hackflare-app

# Run the DNS library tests.
test-dns:
	cargo test -p hackflare-dns

# Run the merged app and DNS tests.
test: test-app test-dns

# Lint the merged application and its test targets.
lint-app:
	cargo clippy -p hackflare-app --all-targets --all-features -- -D warnings

# Lint every workspace crate and its test targets.
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run formatting, linting, and tests.
check: fmt-check lint test

alias run := run-app

# Run the merged application using the repository .env file.
run-app:
	cargo run -p hackflare-app

# Run with an embedded PostgreSQL database.
run-dev:
  cargo run -p hackflare-app -- --dev

# Start PostgreSQL for local application development.
db-up:
	docker compose -f deployment/postgresql.compose.dev.yml up -d

# Stop the development PostgreSQL service.
db-down:
	docker compose -f deployment/postgresql.compose.dev.yml down

# Show development PostgreSQL logs.
db-logs:
	docker compose -f deployment/postgresql.compose.dev.yml logs -f postgres

# Build the merged application image from the repository root.
docker-build-app:
	docker build -f hackflare-app/Dockerfile -t hackflare-app:dev .
