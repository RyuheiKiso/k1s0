# full profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 8GB（CI 検証用、日常開発では非推奨）
# 同梱内容: 全機能（Rust / Go / .NET 8 + MAUI / Node 22 + Playwright / kubectl / helm /
#           argocd / istioctl / kind / kyverno / cosign / syft / opentofu / temporal / dapr /
#           drawio / pandoc）
#
# 用途: GitHub Actions の matrix テストでのみ使用。個人の日常開発では他 9 役のいずれかを使う。

ARG VARIANT=bookworm
FROM mcr.microsoft.com/devcontainers/base:${VARIANT}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

USER root
ENV DEBIAN_FRONTEND=noninteractive

# OS deps
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
        gnupg \
        lsb-release \
        jq \
        shellcheck \
        bash-completion \
        gettext-base \
        pandoc \
        xvfb \
        libgbm1 libxss1 libgtk-3-0 libnss3 libasound2 libxshmfence1 libdrm2 \
        libxcomposite1 libxdamage1 libxrandr2 libxkbcommon0 libatspi2.0-0 libsecret-1-0 \
        fonts-noto-cjk fonts-noto-color-emoji \
    && rm -rf /var/lib/apt/lists/*

# Rust
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=${CARGO_HOME}/bin:${PATH}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal \
    && rustup component add rustfmt clippy rust-analyzer rust-src

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
    && npm install -g pnpm@9.15.0 typescript@5.7.2 @playwright/test@1.49.1 \
        markdownlint-cli2@0.14.0 @mermaid-js/mermaid-cli@11.4.0

# protoc / buf
ARG PROTOC_VERSION=27.3
ARG BUF_VERSION=1.50.1
RUN ARCH="$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch_64/')" \
    && curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-${ARCH}.zip" -o /tmp/protoc.zip \
    && unzip -q /tmp/protoc.zip -d /usr/local \
    && rm /tmp/protoc.zip \
    && curl -fsSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-Linux-$(uname -m)" -o /usr/local/bin/buf \
    && chmod +x /usr/local/bin/buf

# kubectl / helm / kind / istioctl / argocd / kustomize / kyverno / opentofu / k6 / cosign / syft / temporal / flagd
ARG KUBECTL_VERSION=1.31.4
ARG HELM_VERSION=3.16.3
ARG KIND_VERSION=0.26.0
ARG ISTIO_VERSION=1.24.1
ARG ARGOCD_VERSION=2.13.2
ARG KUSTOMIZE_VERSION=5.5.0
ARG KYVERNO_VERSION=1.13.2
ARG TOFU_VERSION=1.8.6
ARG K6_VERSION=0.55.1
ARG COSIGN_VERSION=2.4.1
ARG SYFT_VERSION=1.18.1
ARG TEMPORAL_VERSION=1.1.1
ARG FLAGD_VERSION=0.11.6
ARG YQ_VERSION=4.44.6
RUN curl -fsSL "https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl" -o /usr/local/bin/kubectl && chmod +x /usr/local/bin/kubectl \
    && curl -fsSL "https://get.helm.sh/helm-v${HELM_VERSION}-linux-amd64.tar.gz" | tar -C /tmp -xz && mv /tmp/linux-amd64/helm /usr/local/bin/ \
    && curl -fsSL "https://kind.sigs.k8s.io/dl/v${KIND_VERSION}/kind-linux-amd64" -o /usr/local/bin/kind && chmod +x /usr/local/bin/kind \
    && curl -fsSL "https://github.com/istio/istio/releases/download/${ISTIO_VERSION}/istio-${ISTIO_VERSION}-linux-amd64.tar.gz" | tar -C /tmp -xz && mv /tmp/istio-${ISTIO_VERSION}/bin/istioctl /usr/local/bin/ && rm -rf /tmp/istio-${ISTIO_VERSION} \
    && curl -fsSL "https://github.com/argoproj/argo-cd/releases/download/v${ARGOCD_VERSION}/argocd-linux-amd64" -o /usr/local/bin/argocd && chmod +x /usr/local/bin/argocd \
    && curl -fsSL "https://github.com/kubernetes-sigs/kustomize/releases/download/kustomize%2Fv${KUSTOMIZE_VERSION}/kustomize_v${KUSTOMIZE_VERSION}_linux_amd64.tar.gz" | tar -C /usr/local/bin -xz \
    && curl -fsSL "https://github.com/kyverno/kyverno/releases/download/v${KYVERNO_VERSION}/kyverno-cli_v${KYVERNO_VERSION}_linux_x86_64.tar.gz" | tar -C /tmp -xz && mv /tmp/kyverno /usr/local/bin/kyverno \
    && curl -fsSL "https://github.com/opentofu/opentofu/releases/download/v${TOFU_VERSION}/tofu_${TOFU_VERSION}_linux_amd64.tar.gz" | tar -C /usr/local/bin -xz tofu \
    && curl -fsSL "https://github.com/grafana/k6/releases/download/v${K6_VERSION}/k6-v${K6_VERSION}-linux-amd64.tar.gz" | tar -C /tmp -xz && mv /tmp/k6-v${K6_VERSION}-linux-amd64/k6 /usr/local/bin/k6 && rm -rf /tmp/k6-v${K6_VERSION}-linux-amd64 \
    && curl -fsSL "https://github.com/sigstore/cosign/releases/download/v${COSIGN_VERSION}/cosign-linux-amd64" -o /usr/local/bin/cosign && chmod +x /usr/local/bin/cosign \
    && curl -fsSL "https://raw.githubusercontent.com/anchore/syft/main/install.sh" | sh -s -- -b /usr/local/bin v${SYFT_VERSION} \
    && curl -fsSL "https://github.com/temporalio/cli/releases/download/v${TEMPORAL_VERSION}/temporal_cli_${TEMPORAL_VERSION}_linux_amd64.tar.gz" | tar -C /usr/local/bin -xz temporal \
    && curl -fsSL "https://github.com/open-feature/flagd/releases/download/flagd%2Fv${FLAGD_VERSION}/flagd_${FLAGD_VERSION}_Linux_x86_64.tar.gz" | tar -C /usr/local/bin -xz flagd \
    && curl -fsSL "https://github.com/mikefarah/yq/releases/download/v${YQ_VERSION}/yq_linux_amd64" -o /usr/local/bin/yq && chmod +x /usr/local/bin/yq \
    && curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | /bin/bash

# drawio-desktop
ARG DRAWIO_VERSION=29.7.9
RUN curl -fsSL "https://github.com/jgraph/drawio-desktop/releases/download/v${DRAWIO_VERSION}/drawio-amd64-${DRAWIO_VERSION}.deb" -o /tmp/drawio.deb \
    && apt-get update \
    && apt-get install -y --no-install-recommends /tmp/drawio.deb \
    && rm /tmp/drawio.deb \
    && rm -rf /var/lib/apt/lists/* \
    && printf '#!/usr/bin/env bash\nset -euo pipefail\nexec xvfb-run -a /usr/bin/drawio --no-sandbox --disable-gpu --disable-dev-shm-usage "$@"\n' > /usr/local/bin/drawio-export \
    && chmod +x /usr/local/bin/drawio-export

USER vscode
RUN cargo install --locked cargo-nextest cargo-watch \
    && go install github.com/go-delve/delve/cmd/dlv@latest \
    && go install golang.org/x/tools/cmd/goimports@latest \
    && go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.62.2 \
    && go install google.golang.org/protobuf/cmd/protoc-gen-go@latest \
    && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
