# Use the official Rust Alpine image for building
FROM rust:alpine AS builder-bot

RUN apk update && \
    apk add --no-cache --virtual=build-deps \
        libssl-dev \
        libsqlite3-dev \
        libpng-dev \
        jpeg-dev \
        ca-certificates \
        pkg-config \
        opusfile-dev \
    && rm -rf /var/cache/apk/*

USER root

RUN cargo new --bin kasuki
WORKDIR /kasuki
COPY build.rs ./build.rs
COPY schemas ./schemas
COPY Cargo.toml ./Cargo.toml

RUN cargo build --release

# Clean up unnecessary files and dependencies
RUN rm src/*.rs && \
    rm target/release/kasuki* && \
    rm -rf target/release/build/kasuki*

COPY src ./src
RUN cargo build --release

FROM rust:alpine AS builder-migration

USER root

RUN cargo new --bin migration
WORKDIR /migration
COPY Migration/Cargo.toml ./Cargo.toml

RUN cargo build --release

# Clean up unnecessary files and dependencies
RUN rm src/*.rs && \
    rm target/release/migration*

COPY Migration/src ./src
RUN cargo build --release

# Final stage with Alpine as the base image
FROM alpine AS bot

WORKDIR /kasuki

# Install runtime dependencies for the final image
RUN apk update && \
    apk add --no-cache \
        libssl-dev \
        libsqlite3-dev \
        jpeg-dev \
        libpng16 \
        ca-certificates \
        opusfile \
    && rm -rf /var/cache/apk/*

COPY json /kasuki/json
COPY server_image /kasuki/server_image

# Copy the built binaries from builder stages
COPY --from=builder-bot /kasuki/target/release/kasuki /kasuki/
COPY --from=builder-migration /migration/target/release/migration /kasuki/

# Clean up unnecessary files and directories
RUN rm -rf target/

# Rename the migration binary to match the expected directory structure
RUN mv /kasuki/migration /kasuki/Migration

CMD ["./kasuki"]