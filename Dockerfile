FROM elixir:1.15 AS build

# Install build dependencies
RUN apt-get update \
  && apt-get install -y --no-install-recommends build-essential git curl ca-certificates \
  && rm -rf /var/lib/apt/lists/*

ENV MIX_ENV=prod \
    LANG=C.UTF-8

WORKDIR /app

# Cache Elixir deps
COPY mix.exs mix.lock ./
COPY config ./config

RUN mix local.hex --force && mix local.rebar --force
RUN mix deps.get --only prod
RUN mix deps.compile

# Copy source
COPY . .

# Build assets and release
RUN mix assets.deploy
RUN mix release --overwrite

### Runtime image
FROM debian:bookworm-slim AS app
RUN apt-get update \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8 \
    REPLACE_OS_VARS=true \
    PORT=4000

WORKDIR /app

# Copy the release from the build stage
COPY --from=build /app/_build/prod/rel/hackflare .

EXPOSE 4000

CMD ["bin/hackflare", "start"]
