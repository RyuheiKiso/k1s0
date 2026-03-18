# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context is repo root (to include api/proto and system library dependencies)
# x86_64 ライブラリを直接コピーするためプラットフォームを明示固定
FROM --platform=linux/amd64 rust:1.93-bookworm AS chef

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
COPY regions regions
RUN cargo chef prepare --recipe-path recipe.json --manifest-path regions/business/accounting/server/rust/domain-master/Cargo.toml

# cook ステージ: 依存のみビルド（キャッシュ層）
FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
COPY api/proto api/proto
RUN cargo chef cook --release --recipe-path recipe.json --manifest-path regions/business/accounting/server/rust/domain-master/Cargo.toml

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY api/proto api/proto
COPY regions regions

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release --manifest-path regions/business/accounting/server/rust/domain-master/Cargo.toml --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release --manifest-path regions/business/accounting/server/rust/domain-master/Cargo.toml; \
    fi

FROM busybox:1.36.1-musl AS busybox

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/business/accounting/server/rust/domain-master/target/release/k1s0-domain-master-server /k1s0-domain-master-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-domain-master-server"]
