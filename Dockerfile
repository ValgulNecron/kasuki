# Step 1: Compute a recipe file
FROM rust:1-buster AS planner
WORKDIR app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Step 2: Cache project dependencies
FROM rust:1-buster AS cacher
WORKDIR app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Step 3: Build the binary
FROM rust:1-buster AS builder
WORKDIR app
COPY . .
# Copy over the cached dependencies from above
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin app


FROM debian:buster-slim AS bot

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /kasuki/

COPY lang_file /kasuki/lang_file

COPY --from=builder /kasuki/target/release/kasuki /kasuki/.

CMD ["./kasuki"]