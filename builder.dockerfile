FROM elixir:1.19-slim

ENV MIX_ENV=prod \
    LANG=C.UTF-8 \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# Core build deps
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    git \
    curl \
    libssl-dev \
    pkg-config \
    unzip \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ARG RUST_VERSION=1.92.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    rustup toolchain install ${RUST_VERSION} && \
    rustup default ${RUST_VERSION} && \
    rustup component add rustfmt clippy

# Elixir tooling
RUN mix local.hex --force && \
    mix local.rebar --force

WORKDIR /app

# Prime Hex dependencies from the lockfile (prod env, used for production builds).
COPY mix.exs mix.lock ./
COPY config/ ./config/
RUN mix deps.get

# Bake a Dialyzer PLT into the image so CI's type-check step only has to
# incrementally analyse project modules and any deps added since this image
# was built. We need test-env deps (dialyxir lives there) and stub source
# directories for `mix dialyzer --plt`. The compiled artefacts and stub
# sources are dropped after the PLT is copied to /opt/plts so they don't
# bloat the image or surprise downstream consumers.
RUN mkdir -p lib test/support priv/plts && \
    MIX_ENV=test mix deps.get && \
    MIX_ENV=test mix deps.compile && \
    MIX_ENV=test mix dialyzer --plt && \
    mkdir -p /opt/plts && \
    cp -a priv/plts/. /opt/plts/ && \
    rm -rf lib test priv _build

# Prime Rust crates from the workspace manifests before app source changes.
COPY Cargo.toml Cargo.lock ./
COPY native/core/Cargo.toml ./native/core/Cargo.toml

RUN mkdir -p native/core/src && \
    touch native/core/src/lib.rs && \
    cargo fetch --locked

RUN elixir -v && rustc --version && cargo --version && ls /opt/plts

WORKDIR /toolchain
