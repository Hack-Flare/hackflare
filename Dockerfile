# syntax=docker/dockerfile:1.7

FROM ghcr.io/tainers/hackflare-builder:alpine AS build

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

FROM alpine:3.20 AS app

RUN apk add --no-cache \
    libstdc++ \
    openssl \
    ncurses-libs \
    ca-certificates

ENV LANG=C.UTF-8 \
    PORT=4000

WORKDIR /app

RUN adduser -D -u 1000 appuser
USER appuser

COPY --from=build --chown=appuser:appuser /app/release .

EXPOSE 4000

CMD ["sh", "-c", "bin/hackflare eval \"Hackflare.Release.migrate()\" && exec bin/hackflare start"]