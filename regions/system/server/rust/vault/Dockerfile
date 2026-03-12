FROM rust:1.93-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler cmake build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release -p k1s0-vault-server

FROM busybox:1.36.1-musl AS busybox

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-vault-server /k1s0-vault-server
COPY --from=busybox /bin/busybox /busybox
USER nonroot:nonroot
EXPOSE 8090 50059
ENTRYPOINT ["/k1s0-vault-server"]
