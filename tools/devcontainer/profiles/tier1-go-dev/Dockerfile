# tier1-go-dev profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 3.5GB
# 同梱内容: Go 1.24 / Dapr Go SDK 開発依存 / buf / protoc / delve / golangci-lint /
#           gofumpt / mockgen / kind / dapr CLI / kubectl

ARG GO_VARIANT=1.24-bookworm
FROM mcr.microsoft.com/devcontainers/go:${GO_VARIANT}

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
        pkg-config \
        libssl-dev \
        cmake \
    && rm -rf /var/lib/apt/lists/*

# protoc
ARG PROTOC_VERSION=27.3
RUN ARCH="$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch_64/')" \
    && curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-${ARCH}.zip" -o /tmp/protoc.zip \
    && unzip -q /tmp/protoc.zip -d /usr/local \
    && chmod +x /usr/local/bin/protoc \
    && rm /tmp/protoc.zip

# buf CLI
ARG BUF_VERSION=1.50.1
RUN curl -fsSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-Linux-$(uname -m)" -o /usr/local/bin/buf \
    && chmod +x /usr/local/bin/buf

# kind / kubectl
ARG KIND_VERSION=0.26.0
ARG KUBECTL_VERSION=1.31.4
RUN curl -fsSL "https://kind.sigs.k8s.io/dl/v${KIND_VERSION}/kind-linux-amd64" -o /usr/local/bin/kind \
    && chmod +x /usr/local/bin/kind \
    && curl -fsSL "https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl" -o /usr/local/bin/kubectl \
    && chmod +x /usr/local/bin/kubectl

# Dapr CLI
RUN curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | /bin/bash

USER vscode

# Go ツール一式（Dapr Go SDK / delve / golangci-lint / gofumpt / mockgen / protoc-gen-go(grpc)）
ENV GOPATH=/go
ENV PATH=${GOPATH}/bin:${PATH}
RUN go install github.com/go-delve/delve/cmd/dlv@latest \
    && go install mvdan.cc/gofumpt@latest \
    && go install golang.org/x/tools/cmd/goimports@latest \
    && go install go.uber.org/mock/mockgen@latest \
    && go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.62.2 \
    && go install google.golang.org/protobuf/cmd/protoc-gen-go@latest \
    && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest \
    && go install github.com/bufbuild/buf/cmd/buf@latest

# Dapr Go SDK の事前 fetch（コンパイル時のキャッシュ準備）
RUN GO111MODULE=on go mod download github.com/dapr/go-sdk@latest 2>/dev/null || true
