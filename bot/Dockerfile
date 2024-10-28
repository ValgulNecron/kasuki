FROM rust:slim-bookworm AS builder-bot

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    protobuf-compiler libopus-dev \
    youtube-dl \
    && rm -rf /var/lib/apt/lists/*

USER root

RUN cargo new --bin kasuki
WORKDIR /kasuki

COPY build.rs ./build.rs
COPY proto ./proto
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

RUN cargo new --bin Migration
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
    python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/kasuki/.venv/bin:$PATH"
RUN python3 -m venv .venv
RUN ./.venv/bin/pip3 install yt-dlp
ENV PATH="/kasuki/.venv/bin:$PATH"

COPY json /kasuki/json
COPY server_image /kasuki/server_image
COPY --from=builder-bot /kasuki/target/release/kasuki /kasuki/
COPY --from=builder-migration /migration/target/release/migration /kasuki/

CMD ["./kasuki"]
