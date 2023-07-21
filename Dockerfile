FROM rust:latest

WORKDIR /app

COPY Cargo.toml Cargo.toml

COPY src/main.compile.rs /app/src/main.rs

RUN cargo build --release

RUN rm /app/src/main.rs

RUN rm /app/target/release/kasuki

COPY . .

RUN cargo build --release

CMD ["/app/target/release/kasuki"]