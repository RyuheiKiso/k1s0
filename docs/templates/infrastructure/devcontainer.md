# テンプレート仕様 --- Dev Container

## 概要

k1s0 CLI ひな形生成のDev Containerテンプレート仕様。サービスの `language`、`framework`、および依存インフラ（DB / Kafka / Redis）に応じて、VS Code Dev Container の設定ファイルを自動生成する。

Dev Container の全体設計は [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md) を参照。

## 生成対象

| kind       | 生成有無   |
| ---------- | ---------- |
| `server`   | 生成する   |
| `bff`      | 生成する   |
| `client`   | 生成する   |
| `library`  | 生成する   |
| `database` | 生成する   |

全 kind で Dev Container ファイルを生成する。言語・フレームワーク・依存サービスに応じて features、extensions、forwardPorts、depends_on が変化する。

## 配置パス

生成されるファイルは `.devcontainer/` 直下に配置する。

| ファイル                     | 配置パス                                    |
| ---------------------------- | ------------------------------------------- |
| devcontainer.json            | `.devcontainer/devcontainer.json`            |
| Docker Compose 拡張          | `.devcontainer/docker-compose.extend.yaml`   |
| 初期化スクリプト              | `.devcontainer/post-create.sh`               |

## テンプレートファイル一覧

テンプレートは `CLI/templates/devcontainer/` 配下に配置する。

| テンプレートファイル                | 生成先                                       | 説明                            |
| ----------------------------------- | -------------------------------------------- | ------------------------------- |
| `devcontainer.json.tera`            | `.devcontainer/devcontainer.json`            | Dev Container 本体設定          |
| `docker-compose.extend.yaml.tera`   | `.devcontainer/docker-compose.extend.yaml`   | Compose 拡張（depends_on 等）   |
| `post-create.sh.tera`               | `.devcontainer/post-create.sh`               | コンテナ作成後の初期化スクリプト |

### ディレクトリ構成

```
CLI/
└── templates/
    └── devcontainer/
        ├── devcontainer.json.tera
        ├── docker-compose.extend.yaml.tera
        └── post-create.sh.tera
```

## 使用するテンプレート変数

Dev Container テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名          | 型     | devcontainer.json | compose.extend | post-create.sh | 用途                                          |
| --------------- | ------ | ----------------- | -------------- | -------------- | --------------------------------------------- |
| `service_name`  | String | 用                | 用             | 用             | コンテナ名、ワークスペース識別                |
| `language`      | String | 用                | ---            | 用             | 言語別 features / extensions / ツールの選択（`"go"` / `"rust"` / `"typescript"` / `"dart"`） |
| `framework`     | String | 用                | ---            | 用             | フレームワーク固有の features / extensions    |
| `has_database`  | bool   | 用                | 用             | ---            | DB 関連の forwardPorts / depends_on 追加      |
| `database_type` | String | 用                | 用             | ---            | DB 固有のポート / サービス選択                |
| `has_kafka`     | bool   | 用                | 用             | ---            | Kafka 関連の forwardPorts / depends_on 追加   |
| `has_redis`     | bool   | 用                | 用             | ---            | Redis 関連の forwardPorts / depends_on 追加   |

---

## devcontainer.json テンプレート仕様（devcontainer.json.tera）

### テンプレート内容

```tera
{
  "name": "{{ service_name }}",
  "dockerComposeFile": ["../docker-compose.yaml", "docker-compose.extend.yaml"],
  "service": "devcontainer",
  "workspaceFolder": "/workspace",

  "features": {
{% if language == "go" %}
    "ghcr.io/devcontainers/features/go:1": {
      "version": "1.23"
    },
{% endif %}
{% if language == "rust" %}
    "ghcr.io/devcontainers/features/rust:1": {
      "version": "1.82"
    },
{% endif %}
{% if language == "typescript" or framework == "react" %}
    "ghcr.io/devcontainers/features/node:1": {
      "version": "22"
    },
{% endif %}
{% if language == "dart" or framework == "flutter" %}
    "ghcr.io/devcontainers/features/node:1": {
      "version": "22"
    },
{% endif %}
    "ghcr.io/devcontainers/features/docker-in-docker:2": {},
    "ghcr.io/devcontainers/features/kubectl-helm-minikube:1": {
      "helm": "3.16",
      "minikube": "none"
    }
  },

  "customizations": {
    "vscode": {
      "extensions": [
{% if language == "go" %}
        "golang.go",
{% endif %}
{% if language == "rust" %}
        "rust-lang.rust-analyzer",
{% endif %}
{% if language == "typescript" or framework == "react" %}
        "dbaeumer.vscode-eslint",
        "esbenp.prettier-vscode",
{% endif %}
{% if language == "dart" or framework == "flutter" %}
        "Dart-Code.dart-code",
        "Dart-Code.flutter",
{% endif %}
        "ms-azuretools.vscode-docker",
        "redhat.vscode-yaml",
        "42Crunch.vscode-openapi",
        "zxh404.vscode-proto3",
        "GraphQL.vscode-graphql",
        "eamodio.gitlens"
      ],
      "settings": {
        "editor.formatOnSave": true,
        "editor.codeActionsOnSave": {
          "source.fixAll": "explicit",
          "source.organizeImports": "explicit"
        }{% if language == "go" %},
        "go.lintTool": "golangci-lint",
        "go.lintFlags": ["--fast"],
        "[go]": {
          "editor.defaultFormatter": "golang.go"
        }{% endif %}{% if language == "rust" %},
        "[rust]": {
          "editor.defaultFormatter": "rust-lang.rust-analyzer"
        }{% endif %}{% if language == "typescript" or framework == "react" %},
        "[typescript][typescriptreact]": {
          "editor.defaultFormatter": "esbenp.prettier-vscode"
        }{% endif %}{% if language == "dart" or framework == "flutter" %},
        "[dart]": {
          "editor.defaultFormatter": "Dart-Code.dart-code"
        }{% endif %}
      }
    }
  },

  "forwardPorts": [
    8080,
    50051,
{% if language == "typescript" or framework == "react" %}
    3000,
    5173,
{% endif %}
{% if has_database and database_type == "postgresql" %}
    5432,
{% endif %}
{% if has_database and database_type == "mysql" %}
    3306,
{% endif %}
{% if has_redis %}
    6379,
    6380,
{% endif %}
{% if has_kafka %}
    9092,
    8081,
    8090,
{% endif %}
    16686,
    4317,
    4318,
    3100,
    9090,
    3200,
    8200,
    8180
  ],

  "postCreateCommand": "bash .devcontainer/post-create.sh",
  "remoteUser": "vscode"
}
```

### features の言語別選択

| 言語 / フレームワーク | feature                                          | バージョン |
| --------------------- | ------------------------------------------------ | ---------- |
| Go                    | `ghcr.io/devcontainers/features/go:1`            | 1.23       |
| Rust                  | `ghcr.io/devcontainers/features/rust:1`          | 1.82       |
| TypeScript / React    | `ghcr.io/devcontainers/features/node:1`          | 22         |
| Dart / Flutter        | `ghcr.io/devcontainers/features/node:1`          | 22         |
| 共通 (Docker)         | `ghcr.io/devcontainers/features/docker-in-docker:2` | ---     |
| 共通 (Helm)           | `ghcr.io/devcontainers/features/kubectl-helm-minikube:1` | 3.16 |

### VS Code 拡張機能の言語別選択

| 言語 / フレームワーク | 拡張機能                                     |
| --------------------- | -------------------------------------------- |
| Go                    | `golang.go`                                  |
| Rust                  | `rust-lang.rust-analyzer`                    |
| TypeScript / React    | `dbaeumer.vscode-eslint`, `esbenp.prettier-vscode` |
| Dart / Flutter        | `Dart-Code.dart-code`, `Dart-Code.flutter`   |
| 共通                  | `ms-azuretools.vscode-docker`, `redhat.vscode-yaml`, `42Crunch.vscode-openapi`, `zxh404.vscode-proto3`, `GraphQL.vscode-graphql`, `eamodio.gitlens` |

### forwardPorts の条件別選択

| 条件                                 | ポート              | 用途                |
| ------------------------------------ | ------------------- | ------------------- |
| 常時                                 | 8080                | REST サーバー       |
| 常時                                 | 50051               | gRPC サーバー       |
| `language == "typescript"` or `framework == "react"` | 3000, 5173 | React / Vite dev server |
| `has_database and database_type == "postgresql"` | 5432        | PostgreSQL          |
| `has_database and database_type == "mysql"`       | 3306        | MySQL               |
| `has_redis`                          | 6379, 6380          | Redis / Redis セッション |
| `has_kafka`                          | 9092, 8081, 8090    | Kafka / Schema Registry / Kafka UI |
| 常時                                 | 16686               | Jaeger UI           |
| 常時                                 | 4317, 4318          | Jaeger OTLP (gRPC / HTTP) |
| 常時                                 | 3100                | Loki                |
| 常時                                 | 9090                | Prometheus          |
| 常時                                 | 3200                | Grafana             |
| 常時                                 | 8200                | Vault               |
| 常時                                 | 8180                | Keycloak            |

---

## Docker Compose 拡張テンプレート仕様（docker-compose.extend.yaml.tera）

### テンプレート内容

```tera
services:
  devcontainer:
    image: mcr.microsoft.com/devcontainers/base:ubuntu-24.04
    volumes:
      - ..:/workspace:cached
    command: sleep infinity
{% if has_database or has_kafka or has_redis %}
    depends_on:
{% endif %}
{% if has_database and database_type == "postgresql" %}
      - postgres
{% endif %}
{% if has_database and database_type == "mysql" %}
      - mysql
{% endif %}
{% if has_redis %}
      - redis
{% endif %}
{% if has_kafka %}
      - kafka
{% endif %}
```

### depends_on の条件別選択

| 条件                                           | depends_on サービス |
| ---------------------------------------------- | ------------------- |
| `has_database and database_type == "postgresql"` | `postgres`         |
| `has_database and database_type == "mysql"`     | `mysql`             |
| `has_redis`                                     | `redis`             |
| `has_kafka`                                     | `kafka`             |

---

## 初期化スクリプトテンプレート仕様（post-create.sh.tera）

### テンプレート内容

```tera
#!/bin/bash
# .devcontainer/post-create.sh
# Generated by k1s0 CLI for {{ service_name }}

set -e

{% if language == "go" %}
# Go ツール
go install golang.org/x/tools/cmd/goimports@latest
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@latest
{% endif %}

{% if language == "rust" %}
# Rust コンポーネント
rustup component add clippy rustfmt
{% endif %}

{% if language == "dart" or framework == "flutter" %}
# Flutter SDK
FLUTTER_VERSION="3.24.0"
git clone https://github.com/flutter/flutter.git -b "${FLUTTER_VERSION}" --depth 1 /opt/flutter
export PATH="/opt/flutter/bin:$PATH"
echo 'export PATH="/opt/flutter/bin:$PATH"' >> ~/.bashrc
flutter precache --web
flutter config --no-analytics
{% endif %}

{% if language == "go" %}
# protobuf コンパイラ
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# buf (Protocol Buffers ツール)
BUF_VERSION="1.47.2"
curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" -o /usr/local/bin/buf
chmod +x /usr/local/bin/buf
{% endif %}

echo "Dev Container setup complete for {{ service_name }}."
```

### 言語別インストール内容

| 言語 / フレームワーク | インストール内容                                                                  |
| --------------------- | --------------------------------------------------------------------------------- |
| Go                    | goimports, golangci-lint, protoc-gen-go, protoc-gen-go-grpc, oapi-codegen, protobuf-compiler, buf |
| Rust                  | clippy, rustfmt                                                                   |
| Dart / Flutter        | Flutter SDK 3.24.0, flutter precache, flutter config                              |

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、Dev Container の設定内容が変わる。

| 条件                           | 選択肢                      | devcontainer.json への影響                          | compose.extend への影響        | post-create.sh への影響                     |
| ------------------------------ | --------------------------- | --------------------------------------------------- | ------------------------------ | ------------------------------------------- |
| 言語 (`language`)              | `go`                        | Go feature + extensions + settings 追加             | ---                            | Go ツール + protobuf + buf インストール     |
| 言語 (`language`)              | `rust`                      | Rust feature + extensions + settings 追加           | ---                            | clippy + rustfmt インストール               |
| 言語 (`language`)              | `typescript`                | Node feature + ESLint/Prettier extensions 追加      | ---                            | ---                                         |
| 言語 (`language`)              | `dart`                      | Node feature + Dart/Flutter extensions 追加         | ---                            | Flutter SDK インストール                    |
| フレームワーク (`framework`)   | `react`                     | Node feature + ESLint/Prettier + forwardPorts 3000/5173 | ---                        | ---                                         |
| フレームワーク (`framework`)   | `flutter`                   | Node feature + Dart/Flutter extensions 追加         | ---                            | Flutter SDK インストール                    |
| DB 有無 (`has_database`)       | `true`                      | DB ポートを forwardPorts に追加                     | DB サービスを depends_on に追加 | ---                                         |
| DB 種別 (`database_type`)      | `postgresql`                | 5432 を forwardPorts に追加                         | `postgres` を depends_on に追加 | ---                                         |
| DB 種別 (`database_type`)      | `mysql`                     | 3306 を forwardPorts に追加                         | `mysql` を depends_on に追加   | ---                                         |
| Kafka 有無 (`has_kafka`)       | `true`                      | 9092/8081/8090 を forwardPorts に追加               | `kafka` を depends_on に追加   | ---                                         |
| Redis 有無 (`has_redis`)       | `true`                      | 6379/6380 を forwardPorts に追加                    | `redis` を depends_on に追加   | ---                                         |

---

## 生成例

### Go REST サーバー（PostgreSQL + Redis あり）の場合

入力:
```json
{
  "service_name": "order-api",
  "language": "go",
  "framework": "",
  "has_database": true,
  "database_type": "postgresql",
  "has_kafka": false,
  "has_redis": true
}
```

生成されるファイル:

**`.devcontainer/devcontainer.json`**:
- features: Go 1.23 + Python 3.12 + Docker-in-Docker + kubectl/Helm
- extensions: `golang.go` + 共通拡張機能
- settings: Go 固有設定（golangci-lint, formatOnSave）
- forwardPorts: 8080, 50051, 5432, 6379, 6380, 16686, 4317, 4318, 3100, 9090, 3200, 8200, 8180

**`.devcontainer/docker-compose.extend.yaml`**:
- depends_on: `postgres`, `redis`

**`.devcontainer/post-create.sh`**:
- Go ツール（goimports, golangci-lint, protoc-gen-go 等）
- protobuf-compiler + buf
- pre-commit

### Rust gRPC サーバー（DB なし・Kafka あり）の場合

入力:
```json
{
  "service_name": "event-processor",
  "language": "rust",
  "framework": "",
  "has_database": false,
  "database_type": "",
  "has_kafka": true,
  "has_redis": false
}
```

生成されるファイル:

**`.devcontainer/devcontainer.json`**:
- features: Rust 1.82 + Python 3.12 + Docker-in-Docker + kubectl/Helm
- extensions: `rust-lang.rust-analyzer` + 共通拡張機能
- settings: Rust 固有設定（rust-analyzer formatter）
- forwardPorts: 8080, 50051, 9092, 8081, 8090, 16686, 4317, 4318, 3100, 9090, 3200, 8200, 8180

**`.devcontainer/docker-compose.extend.yaml`**:
- depends_on: `kafka`

**`.devcontainer/post-create.sh`**:
- Rust コンポーネント（clippy, rustfmt）
- pre-commit

### React クライアントの場合

入力:
```json
{
  "service_name": "ledger-app",
  "language": "typescript",
  "framework": "react",
  "has_database": false,
  "database_type": "",
  "has_kafka": false,
  "has_redis": false
}
```

生成されるファイル:

**`.devcontainer/devcontainer.json`**:
- features: Node 22 + Python 3.12 + Docker-in-Docker + kubectl/Helm
- extensions: `dbaeumer.vscode-eslint`, `esbenp.prettier-vscode` + 共通拡張機能
- settings: TypeScript/React 固有設定（Prettier formatter）
- forwardPorts: 8080, 50051, 3000, 5173, 16686, 4317, 4318, 3100, 9090, 3200, 8200, 8180

**`.devcontainer/docker-compose.extend.yaml`**:
- depends_on: なし

**`.devcontainer/post-create.sh`**:
- pre-commit

### Flutter クライアントの場合

入力:
```json
{
  "service_name": "inventory-app",
  "language": "dart",
  "framework": "flutter",
  "has_database": false,
  "database_type": "",
  "has_kafka": false,
  "has_redis": false
}
```

生成されるファイル:

**`.devcontainer/devcontainer.json`**:
- features: Node 22 + Python 3.12 + Docker-in-Docker + kubectl/Helm
- extensions: `Dart-Code.dart-code`, `Dart-Code.flutter` + 共通拡張機能
- settings: Dart 固有設定（dart-code formatter）
- forwardPorts: 8080, 50051, 16686, 4317, 4318, 3100, 9090, 3200, 8200, 8180

**`.devcontainer/docker-compose.extend.yaml`**:
- depends_on: なし

**`.devcontainer/post-create.sh`**:
- Flutter SDK 3.24.0 インストール
- pre-commit

---

## バージョン同期ルール

Dev Container テンプレートで使用する言語・ツールのバージョンは以下のドキュメントと同期する。バージョン更新時は全ドキュメントを同時に変更すること。

| 言語/ツール | バージョン | 同期先                                                                |
| ----------- | ---------- | --------------------------------------------------------------------- |
| Go          | 1.23       | [CI-CD設計](../../infrastructure/cicd/CI-CD設計.md), [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md)    |
| Rust        | 1.82       | [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md), [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md) |
| Node.js     | 22         | [CI-CD設計](../../infrastructure/cicd/CI-CD設計.md), [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md)    |
| Python      | 3.12       | [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md)                               |
| Flutter     | 3.24.0     | [CI-CD設計](../../infrastructure/cicd/CI-CD設計.md), [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md)    |
| Helm        | 3.16       | [CI-CD設計](../../infrastructure/cicd/CI-CD設計.md), [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md)    |
| buf         | 1.47.2     | [CI-CD設計](../../infrastructure/cicd/CI-CD設計.md), [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md)    |

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [devcontainer設計](../../infrastructure/devenv/devcontainer設計.md) --- Dev Container の全体設計
- [テンプレート仕様-CICD](CICD.md) --- CI/CD テンプレート仕様
- [テンプレート仕様-Helm](Helm.md) --- Helm テンプレート仕様
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) --- Docker Compose 設計
- [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md) --- Docker ビルド・タグ・レジストリ
