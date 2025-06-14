FROM rust:slim-bookworm AS builder-bot

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    libopus-dev \
    youtube-dl \
    && rm -rf /var/lib/apt/lists/* \

RUN rustup default nightly

USER root

RUN cargo new --bin kasuki
WORKDIR /kasuki

COPY build.rs ./build.rs
COPY schemas ./schemas
COPY Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs
RUN rm target/release/kasuki*
RUN rm -rf target/release/build/kasuki*

COPY src ./src
RUN cargo build --release

FROM rust:slim-bookworm AS builder-migration

USER root

RUN cargo new --bin migration
WORKDIR /migration

COPY Migration/Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs
RUN rm target/release/migration*

COPY Migration/src ./src
RUN cargo build --release

FROM debian:trixie-slim AS bot
WORKDIR /kasuki

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates libopus-dev \
    && rm -rf /var/lib/apt/lists/*

COPY json /kasuki/json
COPY server_image /kasuki/server_image
COPY --from=builder-bot /kasuki/target/release/kasuki /kasuki/
COPY --from=builder-migration /migration/target/release/migration /kasuki/
RUN mv /kasuki/migration /kasuki/Migration

CMD ["./kasuki"]