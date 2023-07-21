FROM rust:latest

WORKDIR /app

COPY Cargo.toml Cargo.toml

COPY src/main.compile.rs src/main.rs

RUN cargo build --release

COPY . .

RUN cargo build --release

CMD ["/app/target/release/kasuki"]