# Build stage
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.86-bookworm AS builder

# Install protobuf compiler (for tonic-build in build.rs) and
# cmake + build-essential (for rdkafka cmake-build feature)
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire system directory to resolve path dependencies
COPY . .

WORKDIR /app/server/rust/saga
RUN cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/server/rust/saga/target/release/k1s0-saga-server /k1s0-saga-server

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-saga-server"]
