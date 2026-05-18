# tier2 開発コンテナ Dockerfile
# Tiltfile の docker_build() から参照される（devcontainer/Dockerfile パス）
# src/CLAUDE.md: production / development 区別禁止 — セキュリティ・RLS・audit を弱体化しない
FROM ubuntu:24.04

# 非インタラクティブモードで apt-get を実行する（CI 環境での確認プロンプトを抑制する）
ENV DEBIAN_FRONTEND=noninteractive

# 基礎パッケージ群をインストールする（curl / wget / git / build-essential）
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain をインストールする（stable チャネルを使用する）
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Rust の PATH を設定する（以降の RUN 命令で cargo / rustc を使用できるようにする）
ENV PATH="/root/.cargo/bin:${PATH}"

# Go 1.22.5 をインストールする（tier2 Go ライブラリのビルドに使用する）
RUN wget -qO- https://go.dev/dl/go1.22.5.linux-amd64.tar.gz | tar -C /usr/local -xz
# Go の PATH を設定する（以降の RUN 命令で go を使用できるようにする）
ENV PATH="/usr/local/go/bin:${PATH}"

# Node.js 20.x リポジトリを追加する（NodeSource を使用する）
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
# Node.js をインストールする（TypeScript 開発に使用する）
RUN apt-get install -y nodejs && rm -rf /var/lib/apt/lists/*
# pnpm 9 をグローバルインストールする（tier2 TypeScript workspace 管理に使用する）
RUN npm i -g pnpm@9

# .NET 8 SDK を apt からインストールする（C# tier2 ライブラリのビルドに使用する）
RUN apt-get update && apt-get install -y dotnet-sdk-8.0 && rm -rf /var/lib/apt/lists/*

# kubectl をインストールする（Kubernetes クラスタ操作に使用する）
RUN curl -LO "https://dl.k8s.io/release/$(curl -Ls https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" \
    && install -m 755 kubectl /usr/local/bin/ \
    && rm kubectl

# Tilt をインストールする（Tiltfile による開発環境自動化に使用する）
RUN curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

# 作業ディレクトリを /workspace に設定する（tier2 ソースルートをマウントする）
WORKDIR /workspace
