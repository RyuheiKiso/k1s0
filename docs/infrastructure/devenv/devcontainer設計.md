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
      "version": "1.24"
    },
    "ghcr.io/devcontainers/features/rust:1": {
      "version": "1.93"  // Dockerイメージ戦略.md の rust:1.93-bookworm と同期
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
        "eamodio.gitlens",
        "DavidAnson.vscode-markdownlint"
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
  // devcontainer 内の COMPOSE_PROFILES を infra に設定し、infra サービスのみを起動対象とする
  "containerEnv": {
    "COMPOSE_PROFILES": "infra"
  },
  // devcontainer 起動後にインフラ（postgres/redis/kafka/keycloak等）を自動起動する
  "postStartCommand": "docker compose --profile infra up -d",
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
    # infra profile で起動されるサービスが ready になってから devcontainer を起動する
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
      kafka:
        condition: service_healthy
```

## 初期化スクリプト

```bash
#!/bin/bash
# .devcontainer/post-create.sh
# NOTE: Flutter は devcontainer.json の features で既にインストール済み
# このスクリプトでは重複インストールしない

set -e

# Go ツール（バージョン固定で再現性を確保する）
# golangci-lint は ci.yaml の Install golangci-lint ステップと同一バージョンを使用する
go install golang.org/x/tools/cmd/goimports@v0.31.0
go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.64.8
go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.36.3
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.5.1

# oapi-codegen (OpenAPI Go コード生成)
go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@v2.4.1

# Rust コンポーネント
rustup component add clippy rustfmt

# sqlx-cli（データベースマイグレーション管理ツール）
# postgres feature のみビルドして依存を最小化する
if ! command -v sqlx &>/dev/null; then
    cargo install sqlx-cli --no-default-features --features postgres
fi

# protobuf コンパイラ
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# buf (Protocol Buffers ツール)
BUF_VERSION="1.47.2"
if ! command -v buf &>/dev/null || [[ "$(buf --version 2>/dev/null)" != "${BUF_VERSION}" ]]; then
    curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" -o /usr/local/bin/buf
    chmod +x /usr/local/bin/buf
fi

# just（justfile タスクランナー）
if ! command -v just &>/dev/null; then
    curl -sSL https://just.systems/install.sh | bash -s -- --to /usr/local/bin
fi

# pnpm（TypeScript ワークスペース管理）
# corepack は Node.js に同梱されているため別途インストール不要
corepack enable pnpm

echo "Dev Container setup complete."
echo ""

# セットアップ完了後のツール確認
bash .devcontainer/health-check.sh
```

## sparse-checkout との連携

Dev Container は sparse-checkout の設定に関わらず動作する。開発者がチェックアウトしていない階層の依存サービスは、docker-compose のプロファイルにより選択的に起動する（[docker-compose設計](../docker/docker-compose設計.md) を参照）。

## バージョン同期ルール

- **Rust バージョンは [Dockerイメージ戦略](../docker/Dockerイメージ戦略.md) のビルドステージと同期すること。** Dev Container と本番ビルドで異なるバージョンを使用すると、ビルド再現性が損なわれるため、バージョン更新時は両ファイルを同時に変更する。
- **Flutter SDK**: CI/CD ビルドでは `ghcr.io/cirruslabs/flutter:3.24.0` を使用する。Dev Container では `devcontainer.json` の `features` 経由で同一バージョンをインストールする（`ghcr.io/nicknisi/features/flutter:1` version `3.24.0`）。

## リモート共用サーバーでの Dev Container

VS Code Remote SSH で共用開発サーバーに接続した状態で Dev Container を起動することで、サーバーのリソースを活用しつつ統一された開発環境を利用できる。

### 接続フロー

```
[ローカル PC]                [共用開発サーバー]
  VS Code                    Docker Engine
    │                            │
    ├── Remote-SSH 接続 ──────>  │
    │                            │
    ├── Dev Container 起動 ───>  devcontainer (コンテナ)
    │                            ├── Go, Rust, Node, Flutter
    │                            ├── VS Code Server
    │                            └── docker compose (infra)
    │                            │
    └── ポートフォワード <─────  localhost:5432, 8083, ...
```

### セットアップ手順

1. VS Code に Remote-SSH 拡張と Dev Containers 拡張をインストール
2. Remote-SSH でサーバーに接続（`Remote-SSH: Connect to Host...`）
3. サーバー上のリポジトリを開く
4. コマンドパレットから `Dev Containers: Reopen in Container` を実行
5. `.env` に `COMPOSE_PROJECT_NAME` を設定してサービスを起動

```bash
# サーバー上で .env を設定
cp .env.example .env
echo "COMPOSE_PROJECT_NAME=$(whoami)" >> .env

# infra プロファイルを起動
docker compose --profile infra up -d
```

### 注意事項

- Dev Container はサーバーの Docker Engine 上で動作するため、ビルドキャッシュやボリュームはサーバー上に保存される
- `COMPOSE_PROJECT_NAME` を設定して、他の開発者のコンテナと分離すること
- Docker-in-Docker feature が有効なので、Dev Container 内から `docker compose` を実行できる
- 詳細は [共用開発サーバー設計](共用開発サーバー設計.md) を参照

## 関連ドキュメント

- [コーディング規約](../../architecture/conventions/コーディング規約.md)
- [CI-CD設計](../cicd/CI-CD設計.md)
- [docker-compose設計](../docker/docker-compose設計.md)
- [Dockerイメージ戦略](../docker/Dockerイメージ戦略.md)
- [TauriGUI設計](../../cli/gui/TauriGUI設計.md)
- [共用開発サーバー設計](共用開発サーバー設計.md)
