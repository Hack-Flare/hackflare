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
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix deps.get --only prod

COPY config/ ./config/
COPY native/ ./native/

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix deps.compile

COPY priv/ ./priv/
COPY lib/ ./lib/
COPY assets/ ./assets/
COPY doc/ ./doc/

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix compile

RUN mix assets.deploy

# Ensure generated ExDoc HTML is packaged into `priv/static/docs`
# so Plug.Static (served from priv/static) will serve it in the release.
RUN mkdir -p priv/static/docs && cp -r doc/. priv/static/docs/

RUN mix release --overwrite && \
    cp -r _build/prod/rel/hackflare ./release && \
    mkdir -p release/lib/hackflare-*/priv/static/docs && \
    cp -r doc/. release/lib/hackflare-*/priv/static/docs/

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
COPY --from=build --chown=appuser:appuser /app/doc ./doc

EXPOSE 4000
CMD ["bin/hackflare", "start"]