# syntax=docker/dockerfile:1.7

ARG BUILDPLATFORM

FROM --platform=$BUILDPLATFORM debian:bookworm-slim

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update && \
    apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ARG RUST_VERSION=1.92.0

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    rustup toolchain install ${RUST_VERSION} && \
    rustup default ${RUST_VERSION} && \
    rustup component add clippy rustfmt