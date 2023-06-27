FROM rust:latest

WORKDIR /app

COPY Cargo.toml .

RUN cargo build --release

COPY . .

RUN cargo build --release

CMD ["/app/target/release/kasuki"]