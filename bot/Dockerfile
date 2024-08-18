FROM rust:slim-bookworm AS builder-bot

RUN USER=root cargo new --bin kasuki
WORKDIR /kasuki

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    protobuf-compiler libopus-dev \
    youtube-dl \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./Cargo.toml
COPY proto ./proto
COPY schemas ./schemas
COPY build.rs ./build.rs

RUN cargo build --release

RUN rm src/*.rs

RUN rm target/release/deps/kasuki*
RUN rm target/release/kasuki*

COPY src ./src

RUN cargo build --release

FROM rust:slim-bookworm AS builder-migration

RUN USER=root cargo new --bin migration
WORKDIR /migration

COPY Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm target/release/deps/kasuki*
RUN rm target/release/kasuki*

COPY src ./src

RUN cargo build --release

RUN rm src/*.rs
FROM debian:trixie-slim AS bot

LABEL maintainer="valgul"
LABEL author="valgul"

WORKDIR /kasuki/

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates libopus-dev \
    python3-pip python3 \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

RUN python3 -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

RUN pip3 install youtube-dl

COPY json /kasuki/json
COPY server_image /kasuki/server_image

COPY --from=builder-bot /kasuki/target/release/kasuki /kasuki/
COPY --from=builder-migration /migration/target/release/migration /kasuki/

CMD ["./kasuki"]
