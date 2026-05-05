# syntax=docker/dockerfile:1.7

FROM ghcr.io/tainers/hackflare-builder AS build

WORKDIR /app

COPY mix.exs mix.lock ./
COPY config config

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix deps.get --only prod

COPY native native

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix deps.compile

COPY lib lib
COPY priv priv
COPY assets assets
COPY doc doc

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix compile

RUN mix assets.deploy

RUN mkdir -p priv/static/docs && \
    cp -r doc/. priv/static/docs/

RUN mix release --overwrite && \
    cp -r _build/prod/rel/hackflare /app/release

FROM debian:bookworm-slim AS app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libstdc++6 \
    libssl3 \
    ncurses-bin \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8 \
    PORT=4000

WORKDIR /app

RUN adduser --disabled-password --uid 1000 appuser
USER appuser

COPY --from=build /app/release .

EXPOSE 4000

CMD ["sh", "-c", "bin/hackflare eval \"Hackflare.Release.migrate()\" && exec bin/hackflare start"]