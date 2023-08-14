FROM rust:latest AS builder

RUN USER=root cargo new --bin kasuki

RUN apt-get update && apt-get upgrade -y

WORKDIR /kasuki

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/kasuki*
RUN cargo build --release

FROM ubuntu:23.04

RUN apt-get update && apt-get install -y \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev

WORKDIR /kasuki/

COPY lang_file /kasuki/lang_file

COPY --from=builder /kasuki/target/release/kasuki /kasuki/.

CMD ["./kasuki"]