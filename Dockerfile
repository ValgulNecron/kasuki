# Stage 1: Build stage
FROM rust:1.74-alpine3.18 AS builder

RUN USER=root cargo new --bin kasuki

WORKDIR /kasuki

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/kasuki*
RUN cargo build --release

# Stage 2: Runtime stage
FROM alpine:3.18 AS bot

LABEL maintainer="valgul"
LABEL author="valgul"
LABEL "com.docker.compose.hide"="true"
LABEL hidden="true"

HEALTHCHECK CMD ps aux | grep kasuki || exit 1

WORKDIR /kasuki/

COPY json /kasuki/json

RUN apk update && apk add --no-cache \
  libssl1.1-dev libsqlite3-dev \
  libpng-dev libjpeg-dev \
  ca-certificates

COPY --from=builder /kasuki/target/release/kasuki /kasuki/.

CMD ["./kasuki"]
