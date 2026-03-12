# Build stage
# Note: build context is repo root (to include api/proto for tonic-build)
FROM rust:1.93-bookworm AS builder

# Install protobuf compiler (for tonic-build in build.rs) and
# cmake + build-essential (for rdkafka cmake-build feature)
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only what's needed: proto files + system workspace
COPY api/proto api/proto
COPY regions/system regions/system

WORKDIR /app/regions/system

RUN cargo build --release -p k1s0-graphql-gateway-server

FROM busybox:1.36.1-musl AS busybox

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/system/target/release/k1s0-graphql-gateway-server /k1s0-graphql-gateway-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080

ENTRYPOINT ["/k1s0-graphql-gateway-server"]
