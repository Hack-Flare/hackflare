FROM ghcr.io/hack-flare/hackflare-builder AS build

ENV MIX_ENV=prod \
    LANG=C.UTF-8

WORKDIR /app

# deps first
COPY mix.exs mix.lock ./
COPY config/ ./config/

RUN mix local.hex --force && mix local.rebar --force && \
    mix deps.get --only prod

# source
COPY . .

RUN mix compile && \
    mix assets.deploy && \
    mix release --overwrite


FROM debian:trixie-slim AS app

RUN apt-get update && \
    apt-get install -y libstdc++6 openssl libncurses6 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8 \
    PORT=4000

WORKDIR /app

RUN useradd -m -u 1000 appuser
RUN chown -R appuser:appuser /app
USER appuser

COPY --from=build --chown=appuser:appuser /app/_build/prod/rel/hackflare .

EXPOSE 4000

CMD ["sh", "-c", "bin/hackflare eval \"Hackflare.Release.migrate()\" && exec bin/hackflare start"]