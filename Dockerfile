FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["/app/target/release/kasuki"]