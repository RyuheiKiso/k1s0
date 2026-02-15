# DevContainer 設計

k1s0 の開発環境を DevContainer で統一し、環境構築の手間をゼロにする。

## 基本方針

- 開発に必要なツール・拡張機能をすべて DevContainer に含める
- ホスト環境に依存しない再現可能な開発環境を提供する
- Docker Compose と連携し、依存サービス（DB・Kafka 等）をワンコマンドで起動する

## devcontainer.json

```jsonc
// .devcontainer/devcontainer.json
{
  "name": "k1s0",
  "dockerComposeFile": ["../docker-compose.yaml", "docker-compose.extend.yaml"],
  "service": "devcontainer",
  "workspaceFolder": "/workspace",

  "features": {
    "ghcr.io/devcontainers/features/go:1": {
      "version": "1.23"
    },
    "ghcr.io/devcontainers/features/rust:1": {
      "version": "stable"
    },
    "ghcr.io/devcontainers/features/node:1": {
      "version": "22"  // CI/CD でも Node 22 に合わせる（CI-CD設計.md 参照）
    },
    "ghcr.io/devcontainers/features/python:1": {
      "version": "3.12"
    },
    "ghcr.io/devcontainers/features/docker-in-docker:2": {},
    "ghcr.io/devcontainers/features/kubectl-helm-minikube:1": {
      "helm": "3.16",
      "minikube": "none"
    }
  },

  "customizations": {
    "vscode": {
      "extensions": [
        // Go
        "golang.go",
        // Rust
        "rust-lang.rust-analyzer",
        // TypeScript / React
        "dbaeumer.vscode-eslint",
        "esbenp.prettier-vscode",
        // Dart / Flutter
        "Dart-Code.dart-code",
        "Dart-Code.flutter",
        // Python
        "ms-python.python",
        "charliermarsh.ruff",
        // 共通
        "ms-azuretools.vscode-docker",
        "redhat.vscode-yaml",
        "42Crunch.vscode-openapi",
        "zxh404.vscode-proto3",
        "GraphQL.vscode-graphql",
        "eamodio.gitlens"
      ],
      "settings": {
        // 保存時フォーマット
        "editor.formatOnSave": true,
        "editor.codeActionsOnSave": {
          "source.fixAll": "explicit",
          "source.organizeImports": "explicit"
        },
        // Go
        "go.lintTool": "golangci-lint",
        "go.lintFlags": ["--fast"],
        "[go]": {
          "editor.defaultFormatter": "golang.go"
        },
        // Rust
        "[rust]": {
          "editor.defaultFormatter": "rust-lang.rust-analyzer"
        },
        // TypeScript
        "[typescript][typescriptreact]": {
          "editor.defaultFormatter": "esbenp.prettier-vscode"
        },
        // Dart
        "[dart]": {
          "editor.defaultFormatter": "Dart-Code.dart-code"
        },
        // Python
        "[python]": {
          "editor.defaultFormatter": "charliermarsh.ruff"
        }
      }
    }
  },

  "forwardPorts": [
    8080,   // サーバー（REST）
    50051,  // サーバー（gRPC）
    3000,   // React dev server
    5173,   // Vite dev server
    5432,   // PostgreSQL
    3306,   // MySQL
    6379,   // Redis
    9092,   // Kafka
    16686,  // Jaeger UI
    3100,   // Loki
    9090    // Prometheus
  ],

  "postCreateCommand": "bash .devcontainer/post-create.sh",
  "remoteUser": "vscode"
}
```

## DevContainer 用 Compose 拡張

```yaml
# .devcontainer/docker-compose.extend.yaml
services:
  devcontainer:
    image: mcr.microsoft.com/devcontainers/base:ubuntu-24.04
    volumes:
      - ..:/workspace:cached
    command: sleep infinity
    depends_on:
      - postgres
      - redis
```

## 初期化スクリプト

```bash
#!/bin/bash
# .devcontainer/post-create.sh

set -e

# Go ツール
go install golang.org/x/tools/cmd/goimports@latest
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# oapi-codegen (OpenAPI Go コード生成)
go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@latest

# Rust コンポーネント
rustup component add clippy rustfmt

# Flutter SDK
FLUTTER_VERSION="3.24.0"
git clone https://github.com/flutter/flutter.git -b "${FLUTTER_VERSION}" --depth 1 /opt/flutter
export PATH="/opt/flutter/bin:$PATH"
echo 'export PATH="/opt/flutter/bin:$PATH"' >> ~/.bashrc
flutter precache --web
flutter config --no-analytics

# Python（E2E テスト）
pip install -r e2e/requirements.txt

# Pre-commit フック
pip install pre-commit
pre-commit install

# protobuf コンパイラ
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# buf (Protocol Buffers ツール)
BUF_VERSION="1.47.2"
curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" -o /usr/local/bin/buf
chmod +x /usr/local/bin/buf

echo "DevContainer setup complete."
```

## sparse-checkout との連携

DevContainer は sparse-checkout の設定に関わらず動作する。開発者がチェックアウトしていない階層の依存サービスは、docker-compose のプロファイルにより選択的に起動する（[docker-compose設計](docker-compose設計.md) を参照）。
