FROM rust:latest

WORKDIR /app

RUN apt update && apt install -y libssl-dev

COPY . .

RUN cargo build --release

CMD ["./target/release/kasuki"]