# このDockerfileはリポジトリルート (.) をビルドコンテキストとする特別な設定が必要
# 理由: api/proto/gen/rust ディレクトリへのアクセスが必要なため
# buf/validate.proto は BSR（Buf Schema Registry）依存のため、protoc によるオンデマンド生成を
# 行わず api/proto/gen/rust/ の生成済み .rs ファイルを使用する（CRIT-004 対応）。
# docker-compose.yaml の context: . および justfile の graphql-gateway 特別処理を参照

# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context is repo root (to include api/proto/gen/rust for pre-generated gRPC code)
# buf/validate.proto は BSR（Buf Schema Registry）依存のため Docker ビルドでは protoc
# によるオンデマンド生成を行わず、api/proto/gen/rust/ の生成済み .rs ファイルを使用する。
# そのため libprotobuf-dev（google/protobuf/struct.proto 等）は不要となる。
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# ダイジェスト固定は .github/workflows/pin-docker-digests.yaml で自動更新される
FROM rust@sha256:7c4ae649a84014c467d79319bbf17ce2632ae8b8be123ac2fb2ea5be46823f31 AS chef

# cargo-chef のインストール（依存キャッシュ用）
RUN cargo install cargo-chef --locked

# cmake + build-essential（rdkafka 用）のインストール
# protobuf-compiler と libprotobuf-dev は不要:
#   build.rs が api/proto/gen/rust/ の生成済みファイルを OUT_DIR にコピーするため
#   protoc によるオンデマンドコンパイルを行わない（buf/validate.proto 依存問題を回避）
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# planner ステージ: 依存情報レシピの生成
# api/proto/gen/rust のみを COPY する（proto ソースは不要、生成済み .rs ファイルを使用）
FROM chef AS planner
COPY api/proto/gen/rust api/proto/gen/rust
COPY regions/system regions/system
WORKDIR /app/regions/system
RUN cargo chef prepare --recipe-path recipe.json

# cook ステージ: 依存のみビルド（キャッシュ層）
# api/proto/gen/rust のみを COPY する（proto ソースは不要）
# CARGO_FEATURES: フィーチャーフラグを cook ステージに渡す（dev-auth-bypass 等のフィーチャーをキャッシュに反映するため）
FROM chef AS cook
ARG CARGO_FEATURES=""
COPY api/proto/gen/rust api/proto/gen/rust
COPY --from=planner /app/regions/system/recipe.json regions/system/recipe.json
WORKDIR /app/regions/system
RUN if [ -n "${CARGO_FEATURES}" ]; then \
        cargo chef cook --release --features "${CARGO_FEATURES}" --recipe-path recipe.json -p k1s0-graphql-gateway-server; \
    else \
        cargo chef cook --release --recipe-path recipe.json -p k1s0-graphql-gateway-server; \
    fi

# builder ステージ: ソースコードのビルド
# WORKDIR は cook から /app/regions/system を継承しているため、
# COPY の宛先に絶対パスを指定して正しい場所にソースを配置する
FROM cook AS builder
COPY regions/system /app/regions/system

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release -p k1s0-graphql-gateway-server --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release -p k1s0-graphql-gateway-server; \
    fi

# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# ダイジェスト固定は .github/workflows/pin-docker-digests.yaml で自動更新される
FROM busybox@sha256:3c6ae8008e2c2eedd141725c30b20d9c36b026eb796688f88205845ef17aa213 AS busybox

# runtime ステージ: 最小実行環境
# セキュリティ: ベースイメージのダイジェスト固定（CI/CDで自動更新推奨）
# ダイジェスト固定は .github/workflows/pin-docker-digests.yaml で自動更新される
FROM gcr.io/distroless/cc-debian12@sha256:7e5b8df2f4d36f5599ef4ab856d7d444922531709becb03f3368c6d797d0a5eb

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/system/target/release/k1s0-graphql-gateway-server /k1s0-graphql-gateway-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080

ENTRYPOINT ["/k1s0-graphql-gateway-server"]
