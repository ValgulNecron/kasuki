FROM rust:latest

WORKDIR /app

RUN rustup target add aarch64-unknown-linux-gnu

COPY . .

RUN cargo update

RUN cargo build --release

CMD ["/app/target/release/kasuki"]