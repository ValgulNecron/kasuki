FROM rust:1.74.1-buster AS builder

WORKDIR /kasuki

COPY ./Cargo.toml ./Cargo.toml

RUN USER=root cargo new --bin kasuki

RUN cargo build --release

COPY ./src ./src

RUN cargo build --release

FROM debian:buster-slim AS bot

LABEL maintainer="valgul"
LABEL author="valgul"
LABEL "com.docker.compose.hide"="true"
LABEL hidden="true"

HEALTHCHECK CMD ps aux | grep kasuki || exit 1

WORKDIR /kasuki/

COPY json /kasuki/json

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /kasuki/target/release/kasuki/kasuki.* /kasuki/.

CMD ["./kasuki"]