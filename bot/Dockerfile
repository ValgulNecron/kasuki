FROM rust:slim-bookworm AS builder

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates pkg-config \
    libopus-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup default nightly && rustup update

WORKDIR /kasuki

# Copy workspace configuration
COPY Cargo.toml ./

# Copy member Cargo.tomls
COPY bot/Cargo.toml bot/Cargo.toml
COPY api-server/Cargo.toml api-server/Cargo.toml
COPY worker/Cargo.toml worker/Cargo.toml
COPY Migration/Cargo.toml Migration/Cargo.toml
COPY shared/Cargo.toml shared/Cargo.toml
COPY kasuki-macros/Cargo.toml kasuki-macros/Cargo.toml

# Create dummy source files for dependency caching
RUN mkdir -p bot/src api-server/src worker/src Migration/src shared/src kasuki-macros/src \
    && echo "fn main() {}" > bot/src/main.rs \
    && echo "fn main() {}" > api-server/src/main.rs \
    && echo "fn main() {}" > worker/src/main.rs \
    && echo "fn main() {}" > Migration/src/main.rs \
    && echo "fn lib() {}" > shared/src/lib.rs \
&& echo "fn lib() {}" > kasuki-macros/src/lib.rs
# Build dependencies
RUN cargo build --release

# Remove dummy files
RUN rm -rf bot/src api-server/src worker/src Migration/src shared/src

# Copy real source code and assets
COPY shared/schemas ./shared/schemas
COPY shared/src ./shared/src
COPY shared/build.rs ./shared/build.rs
COPY translation ./translation

COPY bot/src ./bot/src
COPY bot/build.rs ./bot/build.rs

COPY api-server/src ./api-server/src

COPY worker/src ./worker/src

COPY Migration/src ./Migration/src
COPY kasuki-macros/src ./kasuki-macros/src

# Build everything
RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim
WORKDIR /kasuki

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev libsqlite3-dev \
    libpng-dev libjpeg-dev \
    ca-certificates libopus-dev \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /kasuki/target/release/kasuki /usr/local/bin/kasuki
COPY --from=builder /kasuki/target/release/api-server /usr/local/bin/api-server
COPY --from=builder /kasuki/target/release/worker /usr/local/bin/worker
COPY --from=builder /kasuki/target/release/migration /usr/local/bin/migration
COPY translation ./translation

#COPY json /kasuki/json
#COPY server_image /kasuki/server_image

# Entrypoint
COPY entrypoint.sh /kasuki/entrypoint.sh
RUN chmod +x /kasuki/entrypoint.sh

ENTRYPOINT ["/kasuki/entrypoint.sh"]
CMD ["--bot"]
