# Build stage
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.93-bookworm AS builder

WORKDIR /app

# Copy the entire system directory to resolve path dependencies
COPY . .

RUN cargo build --release -p k1s0-app-registry

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -r app && useradd -r -g app app

COPY --from=builder /app/target/release/k1s0-app-registry /k1s0-app-registry

USER app:app
EXPOSE 8080

ENTRYPOINT ["/k1s0-app-registry"]
