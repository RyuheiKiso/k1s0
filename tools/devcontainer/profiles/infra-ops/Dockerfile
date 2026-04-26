# infra-ops profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 4.5GB
# 同梱内容: kubectl / helm / kustomize / argocd CLI / istioctl / flagd CLI / openTofu / k6 /
#           kind / kyverno CLI / cosign / syft / yq / jq / shellcheck

ARG VARIANT=bookworm
FROM mcr.microsoft.com/devcontainers/base:${VARIANT}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

USER root
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        unzip \
        gnupg \
        lsb-release \
        jq \
        shellcheck \
        bash-completion \
        make \
        gettext-base \
    && rm -rf /var/lib/apt/lists/*

# kubectl
ARG KUBECTL_VERSION=1.31.4
RUN curl -fsSL "https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl" -o /usr/local/bin/kubectl \
    && chmod +x /usr/local/bin/kubectl

# helm
ARG HELM_VERSION=3.16.3
RUN curl -fsSL "https://get.helm.sh/helm-v${HELM_VERSION}-linux-amd64.tar.gz" -o /tmp/helm.tgz \
    && tar -C /tmp -xzf /tmp/helm.tgz \
    && mv /tmp/linux-amd64/helm /usr/local/bin/helm \
    && rm -rf /tmp/helm.tgz /tmp/linux-amd64

# kustomize
ARG KUSTOMIZE_VERSION=5.5.0
RUN curl -fsSL "https://github.com/kubernetes-sigs/kustomize/releases/download/kustomize%2Fv${KUSTOMIZE_VERSION}/kustomize_v${KUSTOMIZE_VERSION}_linux_amd64.tar.gz" -o /tmp/kustomize.tgz \
    && tar -C /usr/local/bin -xzf /tmp/kustomize.tgz \
    && rm /tmp/kustomize.tgz

# argocd CLI
ARG ARGOCD_VERSION=2.13.2
RUN curl -fsSL "https://github.com/argoproj/argo-cd/releases/download/v${ARGOCD_VERSION}/argocd-linux-amd64" -o /usr/local/bin/argocd \
    && chmod +x /usr/local/bin/argocd

# istioctl (Ambient mode)
ARG ISTIO_VERSION=1.24.1
RUN curl -fsSL "https://github.com/istio/istio/releases/download/${ISTIO_VERSION}/istio-${ISTIO_VERSION}-linux-amd64.tar.gz" -o /tmp/istio.tgz \
    && tar -C /tmp -xzf /tmp/istio.tgz \
    && mv /tmp/istio-${ISTIO_VERSION}/bin/istioctl /usr/local/bin/istioctl \
    && rm -rf /tmp/istio.tgz /tmp/istio-${ISTIO_VERSION}

# flagd CLI
ARG FLAGD_VERSION=0.11.6
RUN curl -fsSL "https://github.com/open-feature/flagd/releases/download/flagd%2Fv${FLAGD_VERSION}/flagd_${FLAGD_VERSION}_Linux_x86_64.tar.gz" -o /tmp/flagd.tgz \
    && tar -C /usr/local/bin -xzf /tmp/flagd.tgz flagd \
    && chmod +x /usr/local/bin/flagd \
    && rm /tmp/flagd.tgz

# OpenTofu
ARG TOFU_VERSION=1.8.6
RUN curl -fsSL "https://github.com/opentofu/opentofu/releases/download/v${TOFU_VERSION}/tofu_${TOFU_VERSION}_linux_amd64.tar.gz" -o /tmp/tofu.tgz \
    && tar -C /usr/local/bin -xzf /tmp/tofu.tgz tofu \
    && rm /tmp/tofu.tgz

# k6
ARG K6_VERSION=0.55.1
RUN curl -fsSL "https://github.com/grafana/k6/releases/download/v${K6_VERSION}/k6-v${K6_VERSION}-linux-amd64.tar.gz" -o /tmp/k6.tgz \
    && tar -C /tmp -xzf /tmp/k6.tgz \
    && mv /tmp/k6-v${K6_VERSION}-linux-amd64/k6 /usr/local/bin/k6 \
    && rm -rf /tmp/k6.tgz /tmp/k6-v${K6_VERSION}-linux-amd64

# kind
ARG KIND_VERSION=0.26.0
RUN curl -fsSL "https://kind.sigs.k8s.io/dl/v${KIND_VERSION}/kind-linux-amd64" -o /usr/local/bin/kind \
    && chmod +x /usr/local/bin/kind

# Kyverno CLI (ADR-POL-001)
ARG KYVERNO_VERSION=1.13.2
RUN curl -fsSL "https://github.com/kyverno/kyverno/releases/download/v${KYVERNO_VERSION}/kyverno-cli_v${KYVERNO_VERSION}_linux_x86_64.tar.gz" -o /tmp/kyverno.tgz \
    && tar -C /tmp -xzf /tmp/kyverno.tgz \
    && mv /tmp/kyverno /usr/local/bin/kyverno \
    && rm -rf /tmp/kyverno.tgz /tmp/LICENSE /tmp/README.md

# yq
ARG YQ_VERSION=4.44.6
RUN curl -fsSL "https://github.com/mikefarah/yq/releases/download/v${YQ_VERSION}/yq_linux_amd64" -o /usr/local/bin/yq \
    && chmod +x /usr/local/bin/yq

# cosign / syft (サプライチェーン: ADR-SUP-001)
ARG COSIGN_VERSION=2.4.1
ARG SYFT_VERSION=1.18.1
RUN curl -fsSL "https://github.com/sigstore/cosign/releases/download/v${COSIGN_VERSION}/cosign-linux-amd64" -o /usr/local/bin/cosign \
    && chmod +x /usr/local/bin/cosign \
    && curl -fsSL "https://raw.githubusercontent.com/anchore/syft/main/install.sh" | sh -s -- -b /usr/local/bin v${SYFT_VERSION}

# Dapr CLI (ADR-TIER1-001 / Dapr ファサード層)
RUN curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | /bin/bash

# Temporal CLI（ADR-RULE-002）
ARG TEMPORAL_VERSION=1.1.1
RUN curl -fsSL "https://github.com/temporalio/cli/releases/download/v${TEMPORAL_VERSION}/temporal_cli_${TEMPORAL_VERSION}_linux_amd64.tar.gz" -o /tmp/temporal.tgz \
    && tar -C /usr/local/bin -xzf /tmp/temporal.tgz temporal \
    && rm /tmp/temporal.tgz

USER vscode

# kubectl bash completion
RUN echo 'source <(kubectl completion bash)' >> ~/.bashrc \
    && echo 'alias k=kubectl' >> ~/.bashrc \
    && echo 'complete -F __start_kubectl k' >> ~/.bashrc \
    && echo 'source <(helm completion bash)' >> ~/.bashrc
