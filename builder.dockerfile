FROM elixir:1.19-slim AS builder

RUN apt-get update && \
    apt-get install -y build-essential git curl libssl-dev pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ENV MIX_ENV=prod \
    LANG=C.UTF-8 \
    HEX_HTTP_TIMEOUT=120

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
    mix compile

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    mix deps.compile