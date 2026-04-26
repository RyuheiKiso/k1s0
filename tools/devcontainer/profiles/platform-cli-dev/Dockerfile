# platform-cli-dev profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 3GB
# 同梱内容: Rust Edition 2024 / Node 22（Backstage プラグイン用） / cosign / syft / kubectl /
#           helm / Backstage CLI

ARG VARIANT=1-bookworm
FROM mcr.microsoft.com/devcontainers/rust:${VARIANT}

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

# Node 22 (NodeSource)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/* \
    && npm install -g pnpm@9.15.0 yarn@1.22.22

# kubectl / helm
ARG KUBECTL_VERSION=1.31.4
ARG HELM_VERSION=3.16.3
RUN curl -fsSL "https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl" -o /usr/local/bin/kubectl \
    && chmod +x /usr/local/bin/kubectl \
    && curl -fsSL "https://get.helm.sh/helm-v${HELM_VERSION}-linux-amd64.tar.gz" -o /tmp/helm.tgz \
    && tar -C /tmp -xzf /tmp/helm.tgz \
    && mv /tmp/linux-amd64/helm /usr/local/bin/helm \
    && rm -rf /tmp/helm.tgz /tmp/linux-amd64

# cosign / syft
ARG COSIGN_VERSION=2.4.1
ARG SYFT_VERSION=1.18.1
RUN curl -fsSL "https://github.com/sigstore/cosign/releases/download/v${COSIGN_VERSION}/cosign-linux-amd64" -o /usr/local/bin/cosign \
    && chmod +x /usr/local/bin/cosign \
    && curl -fsSL "https://raw.githubusercontent.com/anchore/syft/main/install.sh" | sh -s -- -b /usr/local/bin v${SYFT_VERSION}

USER vscode

# mold linker
RUN mkdir -p /home/vscode/.cargo \
    && printf '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-C", "link-arg=-fuse-ld=mold"]\n' > /home/vscode/.cargo/config.toml

# Rust components
RUN rustup component add rustfmt clippy rust-analyzer rust-src \
    && cargo install --locked cargo-nextest cargo-watch
