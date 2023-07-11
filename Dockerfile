FROM rust:latest

RUN groupadd -r kasuki && useradd -r -g kasuki kasuki
USER kasuki

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["/app/target/release/kasuki"]