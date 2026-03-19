# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context is repo root (to include system library dependencies)
# x86_64 ライブラリを直接コピーするためプラットフォームを明示固定
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull rust:1.93-bookworm && docker inspect --format="{{index .RepoDigests 0}}" rust:1.93-bookworm
FROM --platform=linux/amd64 rust:1.93-bookworm AS chef

# cargo-chef のインストール（依存キャッシュ用）
RUN cargo install cargo-chef --locked

# cmake + build-essential（rdkafka cmake-build 用）のインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# planner ステージ: 依存情報レシピの生成
FROM chef AS planner
COPY regions regions
RUN cargo chef prepare --recipe-path recipe.json --manifest-path regions/service/inventory/server/rust/inventory/Cargo.toml

# cook ステージ: 依存のみビルド（キャッシュ層）
FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
COPY regions regions
RUN cargo chef cook --release --recipe-path recipe.json --manifest-path regions/service/inventory/server/rust/inventory/Cargo.toml

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY regions regions

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release --manifest-path regions/service/inventory/server/rust/inventory/Cargo.toml --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release --manifest-path regions/service/inventory/server/rust/inventory/Cargo.toml; \
    fi

# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull busybox:1.36.1-musl && docker inspect --format="{{index .RepoDigests 0}}" busybox:1.36.1-musl
FROM busybox:1.36.1-musl AS busybox

# Runtime stage
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull gcr.io/distroless/cc-debian12:nonroot && docker inspect --format="{{index .RepoDigests 0}}" gcr.io/distroless/cc-debian12:nonroot
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/service/inventory/server/rust/inventory/target/release/k1s0-inventory-server /k1s0-inventory-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8311

ENTRYPOINT ["/k1s0-inventory-server"]
