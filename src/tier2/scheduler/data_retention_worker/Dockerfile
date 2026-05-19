# data-retention-worker コンテナイメージ
# マルチステージビルド: rust:1.78-alpine でビルドして distroless/static-debian11 で実行する

# ────────────────────────────────────────────────────
# ステージ 1: ビルダー（rust:1.78-alpine を使用する）
# ────────────────────────────────────────────────────
# rust:1.78-alpine をベースにした静的リンクビルド環境を使用する
FROM rust:1.78-alpine AS builder

# musl-dev: Alpine の C 標準ライブラリ（静的リンク用ヘッダ / ライブラリを提供する）
# openssl-dev / openssl-libs-static: TLS 対応（sqlx runtime-tokio-native-tls で必要）
# pkgconfig: OpenSSL のビルドシステム統合に使用する
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# x86_64-unknown-linux-musl ターゲットを追加して完全静的バイナリを生成できるようにする
RUN rustup target add x86_64-unknown-linux-musl

# OpenSSL を静的リンクするための環境変数を設定する
ENV OPENSSL_STATIC=1

# ビルド作業ディレクトリを設定する
WORKDIR /build

# 依存関係キャッシュのために Cargo.toml を先にコピーしてダミーソースでビルドする
COPY Cargo.toml ./

# ダミーの main.rs を作成してクレート依存関係のみを先にビルドする（Docker レイヤーキャッシュを活用する）
RUN mkdir -p src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl || true

# 実際のソースコードをコピーする
COPY src/ ./src/

# ダミービルドキャッシュを無効化するためにタッチする
RUN touch src/main.rs

# リリースビルドを実行して静的バイナリを生成する
RUN cargo build --release --target x86_64-unknown-linux-musl

# ────────────────────────────────────────────────────
# ステージ 2: ランタイム（distroless/static-debian11 を使用する）
# ────────────────────────────────────────────────────
# distroless/static-debian11: シェルなし・パッケージマネージャなし・最小攻撃面のイメージ
FROM gcr.io/distroless/static-debian11

# ビルド済み静的バイナリをランタイムイメージにコピーする
COPY --from=builder \
    /build/target/x86_64-unknown-linux-musl/release/data-retention-worker \
    /usr/local/bin/data-retention-worker

# 非 root ユーザー（UID 65534 = nobody）でコンテナを実行する
USER 65534

# エントリポイントをバイナリに設定する（CMD なし / 全引数をバイナリに渡す）
ENTRYPOINT ["/usr/local/bin/data-retention-worker"]
