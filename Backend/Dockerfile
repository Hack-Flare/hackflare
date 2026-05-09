FROM ghcr.io/hack-flare/hackflare-builder AS build

WORKDIR /app/Backend
COPY Backend ./

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

COPY --from=build /app/Backend/target/release/hackflare /app/hackflare

USER appuser

EXPOSE 4000

CMD ["./hackflare"]