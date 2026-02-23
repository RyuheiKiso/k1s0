FROM rust:1.88-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler cmake build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
WORKDIR /app/server/rust/scheduler
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/server/rust/scheduler/target/release/k1s0-scheduler-server /k1s0-scheduler-server
USER nonroot:nonroot
EXPOSE 8093
ENTRYPOINT ["/k1s0-scheduler-server"]
