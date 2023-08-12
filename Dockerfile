FROM rust:latest AS builder

RUN USER=root cargo new --bin kasuki
WORKDIR /kasuki

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/kasuki*
RUN cargo build --release

FROM ubuntu:20.04

WORKDIR /kasuki/

COPY lang_file /kasuki/
COPY --from=builder /kasuki/target/release/kasuki/* /kasuki/.

CMD ["./kasuki"]