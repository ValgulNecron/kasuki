FROM rust:slim-bookworm AS builder

RUN USER=root cargo new --bin kasuki

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /kasuki

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/kasuki*
RUN cargo build --release

FROM debian:trixie-slim AS bot

LABEL maintainer="valgul"
LABEL author="valgul"

RUN useradd -m kasuki

WORKDIR /kasuki/

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

USER kasuki

COPY json /kasuki/json

COPY server_image /kasuki/server_image

COPY --from=builder /kasuki/target/release/kasuki/ /kasuki/

CMD ["./kasuki"]