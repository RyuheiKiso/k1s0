# project-master サーバービルド
# cargo-chef でレイヤーキャッシュを最大化し、distroless イメージで最小化する

FROM --platform=linux/amd64 rust:1.93-bookworm AS chef
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y cmake build-essential protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app

FROM chef AS planner
COPY regions regions
RUN cargo chef prepare --recipe-path recipe.json \
    --manifest-path regions/business/taskmanagement/server/rust/project-master/Cargo.toml

FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
COPY regions regions
RUN cargo chef cook --release --recipe-path recipe.json \
    --manifest-path regions/business/taskmanagement/server/rust/project-master/Cargo.toml

FROM cook AS builder
COPY regions regions
COPY api api
RUN cargo build --release \
    --manifest-path regions/business/taskmanagement/server/rust/project-master/Cargo.toml

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/regions/business/taskmanagement/server/rust/project-master/target/release/k1s0-project-master-server /k1s0-project-master-server

USER nonroot:nonroot
EXPOSE 8210

ENTRYPOINT ["/k1s0-project-master-server"]
