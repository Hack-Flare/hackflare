FROM ghcr.io/hack-flare/hackflare-builder AS build

WORKDIR /app/backend
COPY backend ./

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    libstdc++6 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN useradd -m appuser

COPY --from=build /app/backend/target/release/backend /app/backend

USER appuser

EXPOSE 4000

CMD ["./backend"]