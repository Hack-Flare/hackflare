FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
ENV SQLX_OFFLINE_DIR=database/.sqlx

COPY --from=planner /app/recipe.json recipe.json
ARG TARGETARCH
RUN case "$TARGETARCH" in \
        amd64) RUST_TARGET=x86_64-unknown-linux-musl ;; \
        arm64) RUST_TARGET=aarch64-unknown-linux-musl ;; \
        *) echo "unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    RUSTFLAGS="-C target-feature=+crt-static" \
    cargo chef cook --release --target "$RUST_TARGET" --recipe-path recipe.json

COPY . .
ARG TARGETARCH
RUN case "$TARGETARCH" in \
        amd64) RUST_TARGET=x86_64-unknown-linux-musl ;; \
        arm64) RUST_TARGET=aarch64-unknown-linux-musl ;; \
        *) echo "unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target "$RUST_TARGET" --package hackflare-app --bin hackflare-app && \
    cp "target/$RUST_TARGET/release/hackflare-app" /hackflare-app

FROM alpine:3.20

RUN apk add --no-cache ca-certificates \
    && adduser -D -H -s /sbin/nologin hackflare

COPY --from=builder /hackflare-app /hackflare-app
COPY --from=builder /app/database/migrations /app/migrations
COPY --from=builder /app/hackflare-app/static /app/static

ENV MIGRATIONS_PATH=/app/migrations
ENV FRONTEND_STATIC_DIR=/app/static

USER hackflare
EXPOSE 8080/tcp 5353/udp
ENTRYPOINT ["/hackflare-app"]
