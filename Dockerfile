FROM rust:latest AS builder

RUN USER=root cargo new --bin kasuki
WORKDIR /kasuki

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/kasuki*
RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /kasuki/target/release/kasuki .

CMD ["./kasuki"]