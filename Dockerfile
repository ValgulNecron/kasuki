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

RUN apt-get update && \
    apt-get install -y libssl1.1

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget \
        ca-certificates && \
    wget https://github.com/sgerrand/alpine-pkg-glibc/releases/download/2.29-r0/glibc-2.29-r0.deb && \
    dpkg -i glibc-2.29-r0.deb && \
    rm glibc-2.29-r0.deb && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /kasuki/

COPY --from=builder /kasuki/target/release/kasuki /kasuki/.

CMD ["./kasuki"]