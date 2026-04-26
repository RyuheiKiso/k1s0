# tier3-native-dev profile (Phase 2)
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# サイズ目標: 約 5.5GB
# 同梱内容: .NET 8 SDK / MAUI workload / Android SDK 最小セット / WinUI 開発資材
#
# 注: Linux 上の Android emulator は amd64 + KVM 前提、WinUI / iOS のフル開発は別ホスト
# が必要となる。本コンテナは「ビルド・テスト経路の整備」を主目的とし、Android emulator は
# 個別 Runbook（docs/40_運用ライフサイクル/）に委譲する。

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
        zlib1g-dev \
        openjdk-17-jdk-headless \
        libxext6 \
        libxi6 \
        libxrender1 \
        libxtst6 \
        libfreetype6 \
        libfontconfig1 \
    && rm -rf /var/lib/apt/lists/*

ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64

# Android command-line tools 最小セット
ARG ANDROID_CMDLINE_VERSION=11076708
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_SDK_ROOT=${ANDROID_HOME}
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools \
    && curl -fsSL "https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_CMDLINE_VERSION}_latest.zip" -o /tmp/cmdtools.zip \
    && unzip -q /tmp/cmdtools.zip -d ${ANDROID_HOME}/cmdline-tools \
    && mv ${ANDROID_HOME}/cmdline-tools/cmdline-tools ${ANDROID_HOME}/cmdline-tools/latest \
    && rm /tmp/cmdtools.zip
ENV PATH=${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${PATH}

# Android platforms / build-tools 最小（API 34）
RUN yes | sdkmanager --licenses >/dev/null \
    && sdkmanager \
        "platform-tools" \
        "platforms;android-34" \
        "build-tools;34.0.0" \
        "ndk;26.3.11579264" \
    >/dev/null

USER vscode

# .NET MAUI workload
# Linux ホスト上で動く workload は `maui-android` が中心。`maui` 包括 alias は
# 一部の環境で iOS / Mac Catalyst の依存解決に失敗するため明示的に android のみ install する。
# iOS / WinUI のフルビルドは Mac / Windows 実機を別途必要とする運用前提。
RUN dotnet workload install maui-android --skip-manifest-update \
    && dotnet tool install --global dotnet-format
ENV PATH=/home/vscode/.dotnet/tools:${PATH}
