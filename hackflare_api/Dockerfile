FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
# Install musl toolchain which is required to build with statically linked libc
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
        linux/amd64) RUST_TARGET=x86_64-unknown-linux-musl ;; \
        linux/arm64) RUST_TARGET=aarch64-unknown-linux-musl ;; \
    esac && \
    RUSTFLAGS="-C target-feature=+crt-static" \
    cargo chef cook --release --target "$RUST_TARGET" --recipe-path recipe.json
ENV SQLX_OFFLINE=true
COPY . .
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
        linux/amd64) RUST_TARGET=x86_64-unknown-linux-musl ;; \
        linux/arm64) RUST_TARGET=aarch64-unknown-linux-musl ;; \
    esac && \
    RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target "$RUST_TARGET" --package hackflare-api --bin hackflare-api && \
    cp "target/$RUST_TARGET/release/hackflare-api" /hackflare-api

FROM alpine:3.20
# Install CA certificates for HTTPS support and ensure proper /etc filesystem for DNS
RUN apk add --no-cache ca-certificates
COPY --from=builder /hackflare-api /hackflare-api
ENTRYPOINT ["/hackflare-api"]
