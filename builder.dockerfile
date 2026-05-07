FROM elixir:1.19-slim

# Core build deps
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    git \
    curl \
    libssl-dev \
    pkg-config \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    rustup default stable

# Elixir tooling
RUN mix local.hex --force && \
    mix local.rebar --force

# Ensure consistent locale
ENV LANG=C.UTF-8

RUN elixir -v && rustc --version && cargo --version

WORKDIR /toolchain