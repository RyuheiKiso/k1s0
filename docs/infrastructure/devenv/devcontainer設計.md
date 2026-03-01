# Dev Container 設計

k1s0 の開発環境を Dev Container で統一し、環境構築の手間をゼロにする。

## 基本方針

- 開発に必要なツール・拡張機能をすべて Dev Container に含める
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
      "version": "1.88"  // Dockerイメージ戦略.md の rust:1.88-bookworm と同期
    },
    "ghcr.io/devcontainers/features/node:1": {
      "version": "22"  // CI/CD でも Node 22 に合わせる（CI-CD設計.md 参照）
    },
    "ghcr.io/nicknisi/features/flutter:1": {
      "version": "3.24.0"
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
        }
      }
    }
  },

  "forwardPorts": [
    8080,   // サーバー（REST）
    50051,  // サーバー（gRPC）
    3000,   // React dev server
    5173,   // Vite dev server（React のビルドツールとして Vite を使用）
    5432,   // PostgreSQL
    3306,   // MySQL
    6379,   // Redis
    6380,   // Redis（BFF セッション用）— docker-compose設計.md の redis-session サービスと対応
    9092,   // Kafka
    8081,   // Schema Registry
    16686,  // Jaeger UI
    4317,   // Jaeger OTLP gRPC — OpenTelemetry SDK からのトレース送信用（可観測性設計.md 参照）
    4318,   // Jaeger OTLP HTTP — OpenTelemetry SDK からのトレース送信用
    3100,   // Loki
    9090,   // Prometheus
    3200,   // Grafana
    8090,   // Kafka UI
    8200,   // Vault
    8180    // Keycloak
  ],

  "postCreateCommand": "bash .devcontainer/post-create.sh",
  "remoteUser": "vscode"
}
```

## Dev Container 用 Compose 拡張

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


# Pre-commit フック
pre-commit install

# protobuf コンパイラ
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# buf (Protocol Buffers ツール)
BUF_VERSION="1.47.2"
curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" -o /usr/local/bin/buf
chmod +x /usr/local/bin/buf

# Tauri CLI（TauriGUI設計.md 参照）
cargo install tauri-cli --locked

# Tauri WebView 依存ライブラリ（Linux）
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

echo "Dev Container setup complete."
```

## sparse-checkout との連携

Dev Container は sparse-checkout の設定に関わらず動作する。開発者がチェックアウトしていない階層の依存サービスは、docker-compose のプロファイルにより選択的に起動する（[docker-compose設計](../docker/docker-compose設計.md) を参照）。

## バージョン同期ルール

- **Rust バージョンは [Dockerイメージ戦略](../docker/Dockerイメージ戦略.md) のビルドステージと同期すること。** Dev Container と本番ビルドで異なるバージョンを使用すると、ビルド再現性が損なわれるため、バージョン更新時は両ファイルを同時に変更する。
- **Flutter SDK**: CI/CD ビルドでは `ghcr.io/cirruslabs/flutter:3.24.0` を使用する。Dev Container では公式 Git リポジトリから同一バージョンをインストールする（初期化スクリプト参照）。

## 関連ドキュメント

- [コーディング規約](../../architecture/conventions/コーディング規約.md)
- [CI-CD設計](../cicd/CI-CD設計.md)
- [docker-compose設計](../docker/docker-compose設計.md)
- [Dockerイメージ戦略](../docker/Dockerイメージ戦略.md)
- [TauriGUI設計](../../cli/gui/TauriGUI設計.md)
