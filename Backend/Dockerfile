FROM rust:1.92-bookworm AS build

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /app/target/release/Backend /app/backend

ENV BACKEND_BIND_HOST=0.0.0.0
ENV BACKEND_BIND_PORT=8080

EXPOSE 8080

CMD ["/app/backend"]
