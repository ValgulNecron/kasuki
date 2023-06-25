FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build --release

RUN ls

CMD ["./target/release/kasuki"]