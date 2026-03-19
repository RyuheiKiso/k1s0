# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context must be ./regions/system (to include library dependencies)
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull rust:1.93-bookworm && docker inspect --format="{{index .RepoDigests 0}}" rust:1.93-bookworm
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
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# cook ステージ: 依存のみビルド（キャッシュ層）
FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-event-monitor-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release -p k1s0-event-monitor-server --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release -p k1s0-event-monitor-server; \
    fi

# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull busybox:1.36.1-musl && docker inspect --format="{{index .RepoDigests 0}}" busybox:1.36.1-musl
FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO: ダイジェスト固定 - docker pull gcr.io/distroless/cc-debian12:nonroot && docker inspect --format="{{index .RepoDigests 0}}" gcr.io/distroless/cc-debian12:nonroot
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-event-monitor-server /k1s0-event-monitor-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-event-monitor-server"]
