# Build stage
# Note: build context is repo root (to include api/proto and system library dependencies)
FROM rust:1.93-bookworm AS builder

# Install protobuf compiler (for tonic-build in build.rs) and
# cmake + build-essential (for rdkafka cmake-build feature)
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only what's needed: proto files + regions (system libs + business code)
COPY api/proto api/proto
COPY regions regions

RUN cargo build --release --manifest-path regions/business/accounting/server/rust/domain-master/Cargo.toml

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -r app && useradd -r -g app app

COPY --from=builder /app/regions/business/accounting/server/rust/domain-master/target/release/k1s0-domain-master-server /k1s0-domain-master-server

USER app:app
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-domain-master-server"]
