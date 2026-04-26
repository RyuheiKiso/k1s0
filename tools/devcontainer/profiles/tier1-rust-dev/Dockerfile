# tier1-rust-dev profile (Phase 1 bootstrap)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# 関連 ADR: ADR-DEV-001 (Paved Road), ADR-DEV-002 (WSL2 ネイティブ docker-ce),
#           ADR-TIER1-001 (Go/Rust ハイブリッド), ADR-TIER1-002 (Protobuf gRPC)
#
# Phase 1 簡略化:
# - 設計の正典では tools/devcontainer/base/ (debian slim 起点) から派生する方針だが、
#   Platform/Build チームによる base image の GHCR 配布インフラが未整備のため、
#   bootstrap として Microsoft 公式 mcr.microsoft.com/devcontainers/rust:1-bookworm を起点とする。
# - Phase 2 で tools/devcontainer/base/ を整備した時点で切替える。

ARG VARIANT=1-bookworm
FROM mcr.microsoft.com/devcontainers/rust:${VARIANT}

USER root

ENV DEBIAN_FRONTEND=noninteractive

# k1s0 固有の追加 toolchain:
#   - clang / lld: mold (link driver の代替) と native build 依存
#   - mold: 高速 linker (Rust ビルド時間短縮)
#   - pkg-config / libssl-dev / cmake: 一部 crate (openssl-sys 等) が要求
RUN apt-get update && apt-get install -y --no-install-recommends \
        clang \
        lld \
        mold \
        unzip \
        pkg-config \
        libssl-dev \
        cmake \
    && rm -rf /var/lib/apt/lists/*

# protoc バイナリ導入 (Protocol Buffers 公式リリース)
ARG PROTOC_VERSION=27.3
RUN ARCH="$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch_64/')" \
    && curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-${ARCH}.zip" -o /tmp/protoc.zip \
    && unzip -q /tmp/protoc.zip -d /usr/local \
    && chmod +x /usr/local/bin/protoc \
    && rm /tmp/protoc.zip

# buf CLI (ADR-TIER1-002: Protobuf gRPC 統制)
ARG BUF_VERSION=1.50.1
RUN curl -fsSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-Linux-$(uname -m)" -o /usr/local/bin/buf \
    && chmod +x /usr/local/bin/buf

# kind CLI (Kubernetes in Docker, IMP-DEV-POL-006 ローカル本番再現)
ARG KIND_VERSION=0.26.0
RUN curl -fsSL "https://kind.sigs.k8s.io/dl/v${KIND_VERSION}/kind-linux-amd64" -o /usr/local/bin/kind \
    && chmod +x /usr/local/bin/kind

USER vscode

# mold をデフォルト linker に設定 (cargo の rustflags 経由)
RUN mkdir -p /home/vscode/.cargo \
    && printf '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-C", "link-arg=-fuse-ld=mold"]\n' > /home/vscode/.cargo/config.toml

# rustup component / cargo plugin (Rust Edition 2024 開発の基本セット)
RUN rustup component add rustfmt clippy rust-analyzer rust-src \
    && cargo install --locked cargo-nextest cargo-watch
