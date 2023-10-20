# Step 1: Compute a recipe file
FROM rust:1-buster AS builder
WORKDIR app
COPY . .
RUN cargo run --release

FROM debian:buster-slim AS bot

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /kasuki/

COPY lang_file /kasuki/lang_file

COPY --from=builder /kasuki/target/release/kasuki /kasuki/.

CMD ["./kasuki"]