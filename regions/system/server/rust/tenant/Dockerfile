FROM rust:1.93-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler cmake build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release -p k1s0-tenant-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -r app && useradd -r -g app app
COPY --from=builder /app/target/release/k1s0-tenant-server /k1s0-tenant-server
USER app:app
EXPOSE 8089 50051
ENTRYPOINT ["/k1s0-tenant-server"]
