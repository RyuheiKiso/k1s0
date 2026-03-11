# Build stage
# Note: build context is repo root (to include system library dependencies)
FROM rust:1.93-bookworm AS builder

# Install cmake + build-essential (for rdkafka cmake-build feature)
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only what's needed: regions (system libs + service code)
COPY regions regions

RUN cargo build --release --manifest-path regions/service/order/server/rust/order/Cargo.toml

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/service/order/server/rust/order/target/release/k1s0-order-server /k1s0-order-server

USER nonroot:nonroot
EXPOSE 8310

ENTRYPOINT ["/k1s0-order-server"]
