# tier2-dev profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 5GB
# 同梱内容: .NET 8 SDK / Go 1.24 / Temporal CLI / Dapr Components テスト資材 / kubectl / kind

ARG VARIANT=1-8.0-bookworm
FROM mcr.microsoft.com/devcontainers/dotnet:${VARIANT}

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
    && rm -rf /var/lib/apt/lists/*

# Go 1.24
ARG GO_VERSION=1.24.0
RUN curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" -o /tmp/go.tgz \
    && rm -rf /usr/local/go \
    && tar -C /usr/local -xzf /tmp/go.tgz \
    && rm /tmp/go.tgz
ENV PATH=/usr/local/go/bin:${PATH}
ENV GOPATH=/go
ENV PATH=${GOPATH}/bin:${PATH}

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

# Temporal CLI（ADR-RULE-002）
ARG TEMPORAL_VERSION=1.1.1
RUN curl -fsSL "https://github.com/temporalio/cli/releases/download/v${TEMPORAL_VERSION}/temporal_cli_${TEMPORAL_VERSION}_linux_amd64.tar.gz" -o /tmp/temporal.tgz \
    && tar -C /usr/local/bin -xzf /tmp/temporal.tgz temporal \
    && rm /tmp/temporal.tgz

# kind / kubectl / Dapr CLI（Dapr Components テスト用）
ARG KIND_VERSION=0.26.0
ARG KUBECTL_VERSION=1.31.4
RUN curl -fsSL "https://kind.sigs.k8s.io/dl/v${KIND_VERSION}/kind-linux-amd64" -o /usr/local/bin/kind \
    && chmod +x /usr/local/bin/kind \
    && curl -fsSL "https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl" -o /usr/local/bin/kubectl \
    && chmod +x /usr/local/bin/kubectl \
    && curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | /bin/bash

USER vscode

# .NET tooling: dotnet-format / dotnet-ef
RUN dotnet tool install --global dotnet-format \
    && dotnet tool install --global dotnet-ef
ENV PATH=/home/vscode/.dotnet/tools:${PATH}

# Go tooling
RUN go install github.com/go-delve/delve/cmd/dlv@latest \
    && go install mvdan.cc/gofumpt@latest \
    && go install golang.org/x/tools/cmd/goimports@latest \
    && go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.62.2
