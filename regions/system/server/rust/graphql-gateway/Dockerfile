# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context is repo root (to include api/proto for tonic-build)
FROM rust:1.93-bookworm AS chef

# cargo-chef のインストール（依存キャッシュ用）
RUN cargo install cargo-chef --locked

# protobuf コンパイラ（tonic-build 用）と cmake + build-essential（rdkafka 用）のインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# planner ステージ: 依存情報レシピの生成
FROM chef AS planner
COPY api/proto api/proto
COPY regions/system regions/system
WORKDIR /app/regions/system
RUN cargo chef prepare --recipe-path recipe.json

# cook ステージ: 依存のみビルド（キャッシュ層）
FROM chef AS cook
COPY api/proto api/proto
COPY --from=planner /app/regions/system/recipe.json regions/system/recipe.json
WORKDIR /app/regions/system
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-graphql-gateway-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY regions/system regions/system

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release -p k1s0-graphql-gateway-server --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release -p k1s0-graphql-gateway-server; \
    fi

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/system/target/release/k1s0-graphql-gateway-server /k1s0-graphql-gateway-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080

ENTRYPOINT ["/k1s0-graphql-gateway-server"]
