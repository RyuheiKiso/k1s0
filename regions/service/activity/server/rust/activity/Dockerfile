# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context is repo root (to include system library dependencies)
# x86_64 ライブラリを直接コピーするためプラットフォームを明示固定
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO(ops): CI/CDパイプラインでダイジェスト固定を自動化すること（Renovate/Dependabot + docker inspect --format="{{index .RepoDigests 0}}"）
FROM --platform=linux/amd64 rust@sha256:7c4ae649a84014c467d79319bbf17ce2632ae8b8be123ac2fb2ea5be46823f31 AS chef

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
RUN cargo chef prepare --recipe-path recipe.json --manifest-path regions/service/activity/server/rust/activity/Cargo.toml

# cook ステージ: 依存のみビルド（キャッシュ層）
FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
COPY regions regions
RUN cargo chef cook --release --recipe-path recipe.json --manifest-path regions/service/activity/server/rust/activity/Cargo.toml

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY regions regions

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release --manifest-path regions/service/activity/server/rust/activity/Cargo.toml --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release --manifest-path regions/service/activity/server/rust/activity/Cargo.toml; \
    fi

# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO(ops): CI/CDパイプラインでダイジェスト固定を自動化すること（Renovate/Dependabot + docker inspect --format="{{index .RepoDigests 0}}"）
FROM busybox@sha256:3c6ae8008e2c2eedd141725c30b20d9c36b026eb796688f88205845ef17aa213 AS busybox

# Runtime stage
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# TODO(ops): CI/CDパイプラインでダイジェスト固定を自動化すること（Renovate/Dependabot + docker inspect --format="{{index .RepoDigests 0}}"）
FROM gcr.io/distroless/cc-debian12@sha256:7e5b8df2f4d36f5599ef4ab856d7d444922531709becb03f3368c6d797d0a5eb

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/service/activity/server/rust/activity/target/release/k1s0-activity-server /k1s0-activity-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8330

ENTRYPOINT ["/k1s0-activity-server"]
