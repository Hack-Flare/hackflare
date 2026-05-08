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

ARG RUST_VERSION=1.87.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    rustup toolchain install ${RUST_VERSION} && \
    rustup default ${RUST_VERSION} && \
    rustup component add rustfmt clippy

# Elixir tooling
RUN mix local.hex --force && \
    mix local.rebar --force

WORKDIR /app

# Prime Hex dependencies from the lockfile before app source changes.
COPY mix.exs mix.lock ./
COPY config/ ./config/
RUN mix deps.get

# Prime Rust crates from the workspace manifests before app source changes.
COPY Cargo.toml Cargo.lock ./
COPY native/core/Cargo.toml ./native/core/Cargo.toml

RUN mkdir -p native/core/src && \
    touch native/core/src/lib.rs && \
    cargo fetch --locked

RUN elixir -v && rustc --version && cargo --version

WORKDIR /toolchain