# project-master サーバービルド
# cargo-chef でレイヤーキャッシュを最大化し、distroless イメージで最小化する

FROM rust@sha256:7c4ae649a84014c467d79319bbf17ce2632ae8b8be123ac2fb2ea5be46823f31 AS chef
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y cmake build-essential protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app

FROM chef AS planner
COPY regions regions
# cargo chef prepare は --manifest-path 未対応のため、ワークスペースルートに移動して実行する
WORKDIR /app/regions/business
RUN cargo chef prepare --recipe-path /app/recipe.json

FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
COPY regions regions
# ワークスペースルートに移動して依存のみビルドする（キャッシュ層）
WORKDIR /app/regions/business
RUN cargo chef cook --release --recipe-path /app/recipe.json

FROM cook AS builder
# WORKDIR は cook から /app/regions/business を継承しているため、
# COPY の宛先に絶対パスを指定して正しい場所にソースを配置する
COPY regions /app/regions
COPY api /app/api
# ワークスペースルートに移動して本番バイナリをビルドする
WORKDIR /app/regions/business
RUN cargo build --release \
    --manifest-path taskmanagement/server/rust/project-master/Cargo.toml

FROM busybox@sha256:3c6ae8008e2c2eedd141725c30b20d9c36b026eb796688f88205845ef17aa213 AS busybox

FROM gcr.io/distroless/cc-debian12@sha256:7e5b8df2f4d36f5599ef4ab856d7d444922531709becb03f3368c6d797d0a5eb

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
# ワークスペースルート（/app/regions/business）でビルドするため、targetはワークスペースルート直下に生成される
COPY --from=builder /app/regions/business/target/release/k1s0-project-master-server /k1s0-project-master-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8210

ENTRYPOINT ["/k1s0-project-master-server"]
