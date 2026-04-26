# docs-writer profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 1GB（10 役中最軽量）
# 同梱内容: drawio-desktop (xvfb headless) / markdownlint-cli2 / textlint / Pandoc / Mermaid CLI
#
# drawio は CLAUDE.md / docs/CLAUDE.md の規約「md 編集時は drawio で図を書き SVG を埋め込む」
# を満たす必須ツール。Electron アプリのため xvfb-run 経由の headless 化を `drawio-export`
# wrapper スクリプトで提供する。
#
# 軽量を最優先するため Rust / Go / .NET / Web toolchain は含まない。docs 横断調査向けに
# 追加で必要となる toolchain は infra-ops / sdk-dev 等の別プロファイルへ切替えて使う。

ARG NODE_VARIANT=22-bookworm
FROM mcr.microsoft.com/devcontainers/javascript-node:${NODE_VARIANT}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

USER root
ENV DEBIAN_FRONTEND=noninteractive

# OS deps: drawio (Electron 系) と Pandoc / 日本語フォント / 周辺
RUN apt-get update && apt-get install -y --no-install-recommends \
        pandoc \
        xvfb \
        ca-certificates \
        curl \
        git \
        libgbm1 \
        libxss1 \
        libgtk-3-0 \
        libnss3 \
        libasound2 \
        libxshmfence1 \
        libdrm2 \
        libxcomposite1 \
        libxdamage1 \
        libxrandr2 \
        libxkbcommon0 \
        libatspi2.0-0 \
        libsecret-1-0 \
        fonts-noto-cjk \
        fonts-noto-color-emoji \
        graphviz \
        plantuml \
    && rm -rf /var/lib/apt/lists/*

# drawio-desktop を .deb パッケージで導入
ARG DRAWIO_VERSION=29.7.9
RUN curl -fsSL "https://github.com/jgraph/drawio-desktop/releases/download/v${DRAWIO_VERSION}/drawio-amd64-${DRAWIO_VERSION}.deb" -o /tmp/drawio.deb \
    && apt-get update \
    && apt-get install -y --no-install-recommends /tmp/drawio.deb \
    && rm /tmp/drawio.deb \
    && rm -rf /var/lib/apt/lists/*

# drawio headless wrapper
# Usage: drawio-export --export -f svg --transparent input.drawio -o output.svg
RUN printf '#!/usr/bin/env bash\nset -euo pipefail\nexec xvfb-run -a /usr/bin/drawio \\\n    --no-sandbox \\\n    --disable-gpu \\\n    --disable-dev-shm-usage \\\n    "$@"\n' > /usr/local/bin/drawio-export \
    && chmod +x /usr/local/bin/drawio-export

USER node

# 日本語 lint / 図ツール / フォーマッタ
ENV PNPM_HOME=/home/node/.local/share/pnpm
ENV PATH=$PNPM_HOME:$PATH
RUN npm install -g \
        markdownlint-cli2@0.14.0 \
        textlint@14.7.2 \
        textlint-rule-preset-ja-technical-writing@13.0.1 \
        textlint-rule-preset-jtf-style@3.0.2 \
        @mermaid-js/mermaid-cli@11.4.0 \
        prettier@3.6.2

# 動作確認: drawio バージョンを ENV 経由で記録（image build ログで可視化）
RUN drawio-export --version 2>&1 | head -1 || true
