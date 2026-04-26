# sdk-dev profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 5GB
# 同梱内容: 4 言語 SDK ビルド用 (Rust / Go / .NET / Node) を最小プロファイルで束ねる
#
# DS-SW-COMP-132: SDK 4 言語配布、examples / Hello World 経由で間接結合
# 設計の正典では各言語は最小プロファイル（rust-analyzer / go-tools 等の重い IDE 拡張は省く）

ARG VARIANT=bookworm
FROM mcr.microsoft.com/devcontainers/base:${VARIANT}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

USER root
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        unzip \
        git \
        make \
        clang \
        lld \
        mold \
        pkg-config \
        libssl-dev \
        cmake \
    && rm -rf /var/lib/apt/lists/*

# Rust (rustup)
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=${CARGO_HOME}/bin:${PATH}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal \
    && rustup component add rustfmt clippy

# Go 1.24
ARG GO_VERSION=1.24.0
RUN curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" -o /tmp/go.tgz \
    && tar -C /usr/local -xzf /tmp/go.tgz \
    && rm /tmp/go.tgz
ENV PATH=/usr/local/go/bin:${PATH}
ENV GOPATH=/go
ENV PATH=${GOPATH}/bin:${PATH}

# .NET 8 SDK
RUN curl -fsSL https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor -o /usr/share/keyrings/microsoft.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/microsoft.gpg] https://packages.microsoft.com/debian/12/prod bookworm main" > /etc/apt/sources.list.d/microsoft.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends dotnet-sdk-8.0 \
    && rm -rf /var/lib/apt/lists/*

# Node 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/* \
    && npm install -g pnpm@9.15.0

# protoc / buf
ARG PROTOC_VERSION=27.3
ARG BUF_VERSION=1.50.1
RUN ARCH="$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch_64/')" \
    && curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-${ARCH}.zip" -o /tmp/protoc.zip \
    && unzip -q /tmp/protoc.zip -d /usr/local \
    && chmod +x /usr/local/bin/protoc \
    && rm /tmp/protoc.zip \
    && curl -fsSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-Linux-$(uname -m)" -o /usr/local/bin/buf \
    && chmod +x /usr/local/bin/buf

USER vscode
