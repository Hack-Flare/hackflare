FROM elixir:1.19-slim AS build

RUN apt-get update && \
    apt-get install -y build-essential git curl libssl-dev pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ENV MIX_ENV=prod \
    LANG=C.UTF-8 

WORKDIR /app
RUN mix local.hex --force && mix local.rebar --force

COPY mix.exs mix.lock ./
RUN --mount=type=cache,target=/app/deps \
    --mount=type=cache,target=/root/.cargo/registry \
    mix deps.get

COPY config ./config
COPY lib ./lib
COPY native ./native

RUN --mount=type=cache,target=/app/deps \
    --mount=type=cache,target=/app/_build/prod \
    --mount=type=cache,target=/root/.cargo/registry \
    mix deps.compile

COPY assets ./assets
COPY priv ./priv

RUN --mount=type=cache,target=/app/deps \
    --mount=type=cache,target=/app/_build/prod \
    --mount=type=cache,target=/root/.cargo/registry \
    mix compile

COPY . .

RUN --mount=type=cache,target=/app/deps \
    --mount=type=cache,target=/app/_build/prod \
    --mount=type=cache,target=/root/.cargo/registry \
    mix do assets.deploy, release --overwrite && \
    cp -r _build/prod/rel/hackflare ./release

FROM debian:trixie-slim AS app

RUN apt-get update && \
    apt-get install -y libstdc++6 openssl libncurses6 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8 \
    PORT=4000

WORKDIR /app
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

COPY --from=build --chown=appuser:appuser /app/release .

EXPOSE 4000
CMD ["bin/hackflare", "start"]