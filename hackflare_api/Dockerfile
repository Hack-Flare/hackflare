FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
# Install musl toolchain which is required to build with statically linked libc
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN RUSTFLAGS="-C target-feature=+crt-static" \
    cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
ENV SQLX_OFFLINE=true
COPY . .
RUN RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target x86_64-unknown-linux-musl --package hackflare-api --bin hackflare-api

FROM scratch
# Copy CA certificates so that the reqwest HTTP client can make secure HTTPS calls
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/hackflare-api /hackflare-api
ENTRYPOINT ["/hackflare-api"]
