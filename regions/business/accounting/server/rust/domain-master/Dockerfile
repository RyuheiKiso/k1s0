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

FROM busybox:1.36.1-musl AS busybox

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/business/accounting/server/rust/domain-master/target/release/k1s0-domain-master-server /k1s0-domain-master-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-domain-master-server"]
