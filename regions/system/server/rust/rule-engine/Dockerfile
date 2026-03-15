# Build stage
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.93-bookworm AS builder

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

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release -p k1s0-rule-engine-server --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release -p k1s0-rule-engine-server; \
    fi

FROM busybox:1.36.1-musl AS busybox

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-rule-engine-server /k1s0-rule-engine-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-rule-engine-server"]
