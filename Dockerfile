FROM elixir:1.19 AS build

RUN apt-get update \
  && apt-get install -y --no-install-recommends build-essential git curl ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ENV MIX_ENV=prod \
    LANG=C.UTF-8

WORKDIR /app

RUN mix local.hex --force && mix local.rebar --force

COPY mix.exs mix.lock ./
RUN --mount=type=cache,target=/app/deps \
    mix deps.get --only prod

COPY config ./config
COPY lib ./lib
COPY native ./native 

RUN --mount=type=cache,target=/app/deps \
    --mount=type=cache,target=/app/_build/prod \
    --mount=type=cache,target=/root/.cargo/registry \
    mix deps.compile && mix compile

COPY assets ./assets
COPY priv ./priv
COPY . .

RUN --mount=type=cache,target=/app/deps \
    --mount=type=cache,target=/app/_build/prod \
    mix do assets.deploy + release --overwrite && \
    cp -r _build/prod/rel/hackflare ./release

FROM debian:bookworm-slim AS app
RUN apt-get update \
  && apt-get install -y --no-install-recommends openssl ca-certificates libstdc++6 \
  && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8 \
    PORT=4000

WORKDIR /app

RUN chown -R 1000:1000 /app
USER 1000

COPY --from=build --chown=1000:1000 /app/release .

EXPOSE 4000

CMD ["bin/hackflare", "start"]