# tier3-web-dev profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 4.5GB
# 同梱内容: Node 22 / pnpm 9 / Playwright + Chromium / TypeScript / React DevTools / Vite / Vitest

ARG NODE_VARIANT=22-bookworm
FROM mcr.microsoft.com/devcontainers/javascript-node:${NODE_VARIANT}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

USER root
ENV DEBIAN_FRONTEND=noninteractive

# Playwright Chromium / WebKit / Firefox の依存
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        unzip \
        fonts-noto-cjk \
        fonts-noto-color-emoji \
        libnspr4 \
        libnss3 \
        libatk1.0-0 \
        libatk-bridge2.0-0 \
        libcups2 \
        libdrm2 \
        libxkbcommon0 \
        libxcomposite1 \
        libxdamage1 \
        libxfixes3 \
        libxrandr2 \
        libgbm1 \
        libpango-1.0-0 \
        libcairo2 \
        libasound2 \
        libwoff1 \
        libharfbuzz-icu0 \
        libwebpdemux2 \
        libenchant-2-2 \
        libsecret-1-0 \
        libhyphen0 \
        libegl1 \
        libgles2 \
        libgudev-1.0-0 \
        libevdev2 \
    && rm -rf /var/lib/apt/lists/*

# pnpm 9
ARG PNPM_VERSION=9.15.0
RUN npm install -g pnpm@${PNPM_VERSION}

# Playwright と各ブラウザ事前インストール（image build 時に確定）
USER node
ENV PNPM_HOME=/home/node/.local/share/pnpm
ENV PATH=${PNPM_HOME}:${PATH}
RUN pnpm config set global-bin-dir ${PNPM_HOME} \
    && pnpm add -g \
        typescript@5.7.2 \
        @playwright/test@1.49.1 \
        eslint@9.17.0 \
        prettier@3.6.2 \
        vitest@2.1.8 \
        vite@6.0.7 \
        turbo@2.3.3
RUN npx playwright install chromium webkit firefox
