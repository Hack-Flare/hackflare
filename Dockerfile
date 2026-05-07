ARG BUILDER_IMAGE=ghcr.io/hack-flare/hackflare:builder
FROM ${BUILDER_IMAGE} AS build

WORKDIR /app
COPY priv/ ./priv/
COPY lib/ ./lib/
COPY assets/ ./assets/
COPY doc/ ./doc/

RUN --mount=type=cache,target=/app/deps \
    --mount=type=cache,target=/app/_build \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix compile

RUN mix assets.deploy

# Ensure generated ExDoc HTML is packaged into `priv/static/docs`
# so Plug.Static (served from priv/static) will serve it in the release.
RUN mkdir -p priv/static/docs && cp -r doc/. priv/static/docs/

RUN mix release --overwrite && \
    cp -r _build/prod/rel/hackflare ./release

FROM debian:trixie-slim AS app

RUN apt-get update && \
    apt-get install -y libstdc++6 openssl libncurses6 ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8 \
    PORT=4000

WORKDIR /app
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

COPY --from=build --chown=appuser:appuser /app/release .

EXPOSE 4000
CMD ["sh", "-c", "bin/hackflare eval \"Hackflare.Release.migrate()\" && exec bin/hackflare start"]