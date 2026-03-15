FROM rust:1.93-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler cmake build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release -p k1s0-ai-gateway-server --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release -p k1s0-ai-gateway-server; \
    fi

FROM busybox:1.36.1-musl AS busybox

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-ai-gateway-server /k1s0-ai-gateway-server
COPY --from=busybox /bin/busybox /busybox
USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/k1s0-ai-gateway-server"]
