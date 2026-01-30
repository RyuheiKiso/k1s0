# テンプレートシステム設計書

## 概要

k1s0 テンプレートシステムは、Tera テンプレートエンジンを使用して、サービスの雛形を生成します。複数の言語・フレームワークに対応し、Clean Architecture に基づいたディレクトリ構造を提供します。

## テンプレート配置

```
CLI/templates/
├── backend-rust/
│   └── feature/          # Rust バックエンドテンプレート
├── backend-go/
│   └── feature/          # Go バックエンドテンプレート
├── backend-csharp/
│   ├── feature/          # C# バックエンドテンプレート
│   └── domain/           # C# ドメインテンプレート
├── backend-python/
│   ├── feature/          # Python バックエンドテンプレート
│   └── domain/           # Python ドメインテンプレート
├── frontend-react/
│   └── feature/          # React フロントエンドテンプレート
└── frontend-flutter/
    └── feature/          # Flutter フロントエンドテンプレート
```

---

## テンプレート変数

### 基本変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `feature_name` | 機能名（kebab-case） | `user-management` |
| `service_name` | サービス名 | `user-management` |
| `language` | 言語 | `rust`, `go`, `csharp`, `python`, `typescript`, `dart` |
| `service_type` | タイプ | `backend`, `frontend` |
| `k1s0_version` | k1s0 バージョン | `0.1.0` |

### 命名規則変換

| 変数名 | 説明 | 例（入力: `user-management`） |
|--------|------|-----|
| `feature_name_snake` | snake_case | `user_management` |
| `feature_name_pascal` | PascalCase | `UserManagement` |

### オプション変数

| 変数名 | 説明 | デフォルト |
|--------|------|-----------|
| `with_grpc` | gRPC API を含める | `false` |
| `with_rest` | REST API を含める | `false` |
| `with_db` | DB マイグレーションを含める | `false` |

---

## テンプレートファイル規則

### 拡張子

- **`.tera`**: Tera テンプレートとして処理され、拡張子が除去されて出力される
- **その他**: そのままコピーされる

### 例

```
テンプレート:
  Cargo.toml.tera        → 出力: Cargo.toml
  src/main.rs.tera       → 出力: src/main.rs
  .gitignore             → 出力: .gitignore（そのままコピー）
```

---

## backend-rust テンプレート

### ディレクトリ構造

```
feature/backend/rust/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── Cargo.toml.tera
├── README.md.tera
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   │   ├── configmap.yaml.tera
│   │   ├── deployment.yaml.tera
│   │   ├── service.yaml.tera
│   │   └── kustomization.yaml.tera
│   └── overlays/
│       ├── dev/
│       │   └── kustomization.yaml.tera
│       ├── stg/
│       │   └── kustomization.yaml.tera
│       └── prod/
│           └── kustomization.yaml.tera
├── proto/
│   └── service.proto.tera
├── openapi/
│   └── openapi.yaml.tera
├── migrations/
│   ├── 0001_initial.up.sql.tera
│   └── 0001_initial.down.sql.tera
└── src/
    ├── main.rs.tera
    ├── application/
    │   ├── mod.rs.tera
    │   ├── services/
    │   │   └── mod.rs.tera
    │   └── usecases/
    │       └── mod.rs.tera
    ├── domain/
    │   ├── mod.rs.tera
    │   ├── entities/
    │   │   └── mod.rs.tera
    │   └── errors/
    │       └── mod.rs.tera
    ├── infrastructure/
    │   └── mod.rs
    └── presentation/
        └── mod.rs
```

### Cargo.toml.tera

```toml
[package]
name = "{{ feature_name }}"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
# Framework crates
k1s0-error = { path = "../../../../framework/backend/rust/crates/k1s0-error" }
k1s0-config = { path = "../../../../framework/backend/rust/crates/k1s0-config" }
k1s0-observability = { path = "../../../../framework/backend/rust/crates/k1s0-observability" }
k1s0-validation = { path = "../../../../framework/backend/rust/crates/k1s0-validation" }
{% if with_grpc %}
k1s0-grpc-server = { path = "../../../../framework/backend/rust/crates/k1s0-grpc-server" }
{% endif %}
k1s0-resilience = { path = "../../../../framework/backend/rust/crates/k1s0-resilience" }

# Runtime
tokio = { version = "1", features = ["full"] }
{% if with_grpc %}
tonic = "0.12"
prost = "0.13"
{% endif %}
{% if with_rest %}
axum = "0.7"
{% endif %}

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
opentelemetry = "0.24"
{% if with_db %}

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
{% endif %}

[dev-dependencies]
tokio-test = "0.4"
```

### main.rs.tera

```rust
//! {{ feature_name }} サービス
//!
//! k1s0 framework を使用した {{ feature_name }} のエントリポイント。

mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;
use clap::Parser;
use tracing::info;
{% if with_rest %}
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
{% endif %}
{% if with_grpc %}
use tonic::transport::Server as TonicServer;
{% endif %}

/// CLI 引数
#[derive(Parser, Debug)]
#[command(name = "{{ feature_name }}")]
struct Args {
    #[arg(long, default_value = "dev")]
    env: String,

    #[arg(long, default_value = "./config")]
    config: String,
{% if with_grpc %}
    #[arg(long, default_value = "50051")]
    grpc_port: u16,
{% endif %}
{% if with_rest %}
    #[arg(long, default_value = "8080")]
    http_port: u16,
{% endif %}
    #[arg(long, default_value = "9090")]
    health_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Observability 初期化
    let _guard = k1s0_observability::init_with_config(
        k1s0_observability::ObservabilityConfig::builder()
            .service_name("{{ feature_name }}")
            .service_version(env!("CARGO_PKG_VERSION"))
            .environment(&args.env)
            .build(),
    )?;

    info!(service = "{{ feature_name }}", "Starting service");

    // 設定読み込み
    let config = k1s0_config::ConfigBuilder::new()
        .add_source(k1s0_config::File::with_name(&format!("{}/{}.yaml", args.config, args.env)))
        .build()?;

    // ヘルスチェックサーバー起動
    let health_registry = Arc::new(k1s0_health::HealthRegistry::new());
    let health_addr = format!("0.0.0.0:{}", args.health_port).parse()?;
    let health_router = k1s0_health::health_router(health_registry.clone());
    let health_server = tokio::spawn(async move {
        info!(addr = %health_addr, "Starting health check server");
        axum::serve(
            tokio::net::TcpListener::bind(health_addr).await.unwrap(),
            health_router,
        ).await.unwrap();
    });

{% if with_rest %}
    // HTTP サーバー起動
    let http_addr = format!("0.0.0.0:{}", args.http_port).parse()?;
    let http_router = Router::new()
        .route("/", get(|| async { "{{ feature_name }} service is running" }))
        .layer(TraceLayer::new_for_http());
    let http_server = tokio::spawn(async move {
        info!(addr = %http_addr, "Starting HTTP server");
        axum::serve(
            tokio::net::TcpListener::bind(http_addr).await.unwrap(),
            http_router,
        ).await.unwrap();
    });
{% endif %}

{% if with_grpc %}
    // gRPC サーバー起動
    let grpc_addr = format!("0.0.0.0:{}", args.grpc_port).parse()?;
    let grpc_server = tokio::spawn(async move {
        info!(addr = %grpc_addr, "Starting gRPC server");
        TonicServer::builder()
            // .add_service(your_service)
            .serve(grpc_addr)
            .await
            .unwrap();
    });
{% endif %}

    // Graceful shutdown
    info!("Service started, waiting for shutdown signal...");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    health_server.abort();
{% if with_rest %}
    http_server.abort();
{% endif %}
{% if with_grpc %}
    grpc_server.abort();
{% endif %}

    k1s0_observability::shutdown();
    info!("Service shutdown complete");
    Ok(())
}
```

---

## backend-go テンプレート

### ディレクトリ構造

```
feature/backend/go/{service_name}/
├── .k1s0/
│   └── manifest.json
├── go.mod
├── go.sum
├── README.md
├── cmd/
│   └── main.go.tera
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   │   ├── configmap.yaml
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   └── kustomization.yaml
│   └── overlays/
│       ├── dev/
│       ├── stg/
│       └── prod/
├── internal/
│   ├── domain/
│   │   ├── entities/
│   │   └── errors/
│   ├── application/
│   │   ├── services/
│   │   └── usecases/
│   ├── presentation/
│   └── infrastructure/
└── proto/
    └── service.proto
```

### go.mod テンプレート

```go
module github.com/your-org/{{ feature_name }}

go {{ "1.22" }}

require (
    // Framework packages
    github.com/your-org/k1s0-go/config v0.1.0
    github.com/your-org/k1s0-go/observability v0.1.0
    github.com/your-org/k1s0-go/validation v0.1.0
{% if with_grpc %}
    google.golang.org/grpc v1.64.0
    google.golang.org/protobuf v1.34.0
{% endif %}
{% if with_rest %}
    github.com/labstack/echo/v4 v4.12.0
{% endif %}
{% if with_db %}
    github.com/jackc/pgx/v5 v5.6.0
{% endif %}
)
```

---

## backend-csharp テンプレート

### ディレクトリ構造

```
feature/backend/csharp/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── {FeatureName}.sln.tera
├── Directory.Build.props.tera
├── Directory.Packages.props.tera
├── .editorconfig
├── README.md.tera
├── Dockerfile.tera
├── .dockerignore
├── config/
│   ├── default.yaml.tera
│   ├── dev.yaml.tera
│   ├── stg.yaml.tera
│   └── prod.yaml.tera
├── deploy/
│   └── base/
│       ├── configmap.yaml.tera
│       ├── deployment.yaml.tera
│       ├── service.yaml.tera
│       └── kustomization.yaml.tera
├── proto/
│   └── service.proto.tera
├── openapi/
│   └── openapi.yaml.tera
├── buf.yaml
├── buf.gen.yaml.tera
├── src/
│   ├── {FeatureName}.Domain/
│   │   ├── {FeatureName}.Domain.csproj.tera
│   │   ├── Entities/.gitkeep
│   │   ├── ValueObjects/.gitkeep
│   │   ├── Repositories/.gitkeep
│   │   └── Services/.gitkeep
│   ├── {FeatureName}.Application/
│   │   ├── {FeatureName}.Application.csproj.tera
│   │   ├── UseCases/.gitkeep
│   │   ├── Services/.gitkeep
│   │   └── DTOs/.gitkeep
│   ├── {FeatureName}.Infrastructure/
│   │   ├── {FeatureName}.Infrastructure.csproj.tera
│   │   ├── Repositories/.gitkeep
│   │   ├── External/.gitkeep
│   │   └── Persistence/.gitkeep
│   └── {FeatureName}.Presentation/
│       ├── {FeatureName}.Presentation.csproj.tera
│       ├── Program.cs.tera
│       ├── Controllers/.gitkeep
│       ├── Grpc/.gitkeep
│       └── Middleware/.gitkeep
└── tests/
    ├── {FeatureName}.Domain.Tests/
    ├── {FeatureName}.Application.Tests/
    └── {FeatureName}.Integration.Tests/
```

### 特徴

- **ASP.NET Core 8.0** ベース
- **Central Package Management** （`Directory.Packages.props`）でバージョン一元管理
- **4プロジェクト構成**: Domain, Application, Infrastructure, Presentation（Clean Architecture）
- **3テストプロジェクト**: Domain.Tests, Application.Tests, Integration.Tests
- **条件付きレンダリング**: `{% if with_grpc %}` で gRPC、`{% if with_db %}` で EF Core 依存を追加
- **Multi-stage Docker build**: SDK イメージでビルド → ASP.NET ランタイムイメージで実行

---

## backend-python テンプレート

### ディレクトリ構造

```
feature/backend/python/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── pyproject.toml.tera
├── README.md.tera
├── Dockerfile.tera
├── .dockerignore
├── config/
│   ├── default.yaml.tera
│   ├── dev.yaml.tera
│   ├── stg.yaml.tera
│   └── prod.yaml.tera
├── deploy/
│   └── base/
│       ├── configmap.yaml.tera
│       ├── deployment.yaml.tera
│       ├── service.yaml.tera
│       └── kustomization.yaml.tera
├── proto/
│   └── service.proto.tera
├── openapi/
│   └── openapi.yaml.tera
├── buf.yaml
├── buf.gen.yaml.tera
├── src/
│   └── {{ feature_name_snake }}/
│       ├── __init__.py
│       ├── main.py.tera
│       ├── domain/
│       │   ├── __init__.py
│       │   ├── entities/
│       │   └── errors/
│       ├── application/
│       │   ├── __init__.py
│       │   ├── services/
│       │   └── usecases/
│       ├── infrastructure/
│       │   ├── __init__.py
│       │   └── repositories/
│       └── presentation/
│           ├── __init__.py
│           ├── grpc/
│           └── rest/
└── tests/
    ├── conftest.py
    └── test_health.py
```

### 特徴

- **FastAPI 0.115+** ベース
- **uv** によるパッケージ管理（`pyproject.toml`）
- **Pydantic v2** でバリデーション・DTO
- **SQLAlchemy 2.0 + asyncpg** で非同期 DB アクセス
- **Ruff** でフォーマット・リント統合
- **mypy** で型チェック
- **pytest + pytest-asyncio + httpx** でテスト
- **条件付きレンダリング**: `{% if with_grpc %}` で gRPC、`{% if with_db %}` で SQLAlchemy 依存を追加
- **Multi-stage Docker build**: Python 3.12 ベースイメージ

---

## frontend-react テンプレート

### ディレクトリ構造

```
feature/frontend/react/{service_name}/
├── .k1s0/
│   └── manifest.json
├── package.json.tera
├── tsconfig.json
├── vite.config.ts
├── README.md
├── public/
│   └── index.html
├── src/
│   ├── main.tsx.tera
│   ├── App.tsx
│   ├── domain/
│   │   ├── entities/
│   │   └── repositories/
│   ├── application/
│   │   ├── services/
│   │   └── usecases/
│   └── presentation/
│       ├── components/
│       ├── pages/
│       └── hooks/
└── deploy/
    ├── base/
    └── overlays/
```

### package.json.tera

```json
{
  "name": "{{ feature_name }}",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint src --ext .ts,.tsx",
    "test": "vitest"
  },
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "react-router-dom": "^6.23.0"
  },
  "devDependencies": {
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.4.0",
    "vite": "^5.2.0",
    "vitest": "^1.6.0"
  }
}
```

---

## frontend-flutter テンプレート

### ディレクトリ構造

```
feature/frontend/flutter/{service_name}/
├── .k1s0/
│   └── manifest.json
├── pubspec.yaml.tera
├── README.md
├── lib/
│   ├── main.dart.tera
│   └── src/
│       ├── domain/
│       │   ├── entities/
│       │   └── repositories/
│       ├── application/
│       │   ├── services/
│       │   └── usecases/
│       └── presentation/
│           ├── widgets/
│           ├── pages/
│           └── providers/
├── test/
└── deploy/
    ├── base/
    └── overlays/
```

### pubspec.yaml.tera

```yaml
name: {{ feature_name_snake }}
description: {{ feature_name_pascal }} Flutter application
publish_to: 'none'
version: 0.1.0

environment:
  sdk: '>=3.3.0 <4.0.0'
  flutter: '>=3.19.0'

dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.5.0
  go_router: ^14.0.0
  freezed_annotation: ^2.4.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^4.0.0
  build_runner: ^2.4.0
  freezed: ^2.5.0

flutter:
  uses-material-design: true
```

---

## Docker テンプレート

各テンプレートは以下の Docker 関連ファイルを生成する：

| ファイル | 説明 | managed |
|---------|------|:-------:|
| `Dockerfile` | Multi-stage ビルド（Mode A: standalone） | ✅ |
| `Dockerfile.monorepo` | Mode B: monorepo root context ビルド | ✅ |
| `.dockerignore` | Docker ビルド除外パターン | ✅ |
| `compose.yaml` | Docker Compose サービス定義 | ✅ |
| `compose.monorepo.yaml` | monorepo mode の Compose 定義 | ✅ |
| `deploy/docker/otel-collector-config.yaml` | OTEL Collector 設定 | ✅ |
| `deploy/docker/nginx.conf` | nginx 設定（フロントエンドのみ） | ✅ |

### Dockerfile 設計方針

- **Multi-stage build**: ビルド環境と実行環境を分離
- **Non-root ユーザー**: `appuser` でアプリケーションを実行
- **HEALTHCHECK**: コンテナヘルスモニタリング
- **プロキシ対応**: `ARG HTTP_PROXY/HTTPS_PROXY/NO_PROXY`
- **`--no-docker`**: Docker ファイル生成のオプトアウト

### ベースイメージ

| テンプレート | Builder | Runtime |
|-------------|---------|---------|
| backend-rust | `rust:1.85-slim` | `debian:bookworm-slim` |
| backend-go | `golang:1.22-bookworm` | `gcr.io/distroless/static-debian12` |
| backend-csharp | `mcr.microsoft.com/dotnet/sdk:8.0` | `mcr.microsoft.com/dotnet/aspnet:8.0` |
| backend-python | `python:3.12-slim` | `python:3.12-slim` |
| frontend-react | `node:20-slim` | `nginx:1.27-alpine` |
| frontend-flutter | `ghcr.io/cirruslabs/flutter:stable` | `nginx:1.27-alpine` |

---

## managed/protected パス

### managed パス（CLI が自動更新）

| テンプレート | managed パス |
|-------------|-------------|
| backend-rust | `deploy/`, `buf.yaml`, `buf.gen.yaml` |
| backend-go | `deploy/`, `buf.yaml`, `buf.gen.yaml` |
| backend-csharp | `deploy/`, `buf.yaml`, `buf.gen.yaml`, `*.csproj` |
| backend-python | `deploy/`, `buf.yaml`, `buf.gen.yaml`, `pyproject.toml` |
| frontend-react | `deploy/` |
| frontend-flutter | `deploy/` |

### protected パス（CLI が変更しない）

| テンプレート | protected パス |
|-------------|---------------|
| backend-rust | `src/domain/`, `src/application/`, `README.md` |
| backend-go | `internal/domain/`, `internal/application/`, `README.md` |
| backend-csharp | `src/*.Domain/`, `src/*.Application/`, `README.md` |
| backend-python | `src/*/domain/`, `src/*/application/`, `README.md` |
| frontend-react | `src/domain/`, `src/application/`, `src/presentation/`, `README.md` |
| frontend-flutter | `lib/src/domain/`, `lib/src/application/`, `lib/src/presentation/`, `README.md` |

### update_policy

| ポリシー | 説明 |
|---------|------|
| `auto` | `k1s0 upgrade` で自動更新 |
| `suggest_only` | 差分を提示するが自動更新しない |
| `protected` | 一切変更しない |

デフォルトの割り当て:

```
deploy/              → auto
buf.yaml             → auto
src/domain/          → protected
src/application/     → protected
README.md            → suggest_only
config/              → suggest_only
```

---

## 条件付きレンダリング

### Tera 構文

```jinja2
{% if with_grpc %}
// gRPC 関連のコード
{% endif %}

{% if with_rest %}
// REST 関連のコード
{% endif %}

{% if with_db %}
// DB 関連のコード
{% endif %}
```

### 例：Cargo.toml

```toml
[dependencies]
{% if with_grpc %}
tonic = "0.12"
prost = "0.13"
{% endif %}

{% if with_rest %}
axum = "0.7"
{% endif %}

{% if with_db %}
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
{% endif %}
```

---

## fingerprint 計算

### 目的

テンプレートの変更を検出し、`k1s0 upgrade` で差分を適用する。

### 算出方法

1. テンプレートディレクトリを再帰的に走査
2. 除外パターンに一致するファイルをスキップ
3. ファイルを相対パスでソート
4. 各ファイルのパスと内容を SHA-256 でハッシュ化

### 除外パターン

```
.git, .svn, .hg           # バージョン管理
target, node_modules, ...  # ビルド成果物
.DS_Store, Thumbs.db      # OS メタデータ
.idea, .vscode            # IDE
.k1s0                     # k1s0 メタデータ
*.pyc, *.log, *.tmp, ...  # 一時ファイル
.env, .env.local          # 環境設定
```

---

## テンプレート追加ガイド

### 新しいテンプレートを追加する手順

1. **ディレクトリ作成**
   ```
   CLI/templates/{template-name}/feature/
   ```

2. **必須ファイルの配置**
   - `.k1s0/manifest.json.tera`
   - メイン設定ファイル（`Cargo.toml.tera`, `package.json.tera` など）
   - エントリーポイント（`main.rs.tera`, `main.go.tera` など）

3. **Clean Architecture 構造の作成**
   ```
   src/
   ├── domain/
   ├── application/
   ├── presentation/
   └── infrastructure/
   ```

4. **ServiceType への追加**
   `CLI/crates/k1s0-cli/src/commands/new_feature.rs` に追加:
   ```rust
   pub enum ServiceType {
       // ...
       #[value(name = "template-name")]
       TemplateName,
   }
   ```

5. **RequiredFiles への追加**
   `CLI/crates/k1s0-generator/src/lint/required_files.rs` に追加

6. **テスト**
   ```bash
   k1s0 new-feature -t template-name -n test-service
   k1s0 lint feature/{type}/{lang}/test-service
   ```

---

## 今後の拡張予定

1. **テンプレートレジストリ**: リモートからテンプレートを取得
2. **カスタムテンプレート**: ユーザー定義テンプレートのサポート
3. **テンプレートバージョニング**: テンプレート自体のバージョン管理
4. **プラグインシステム**: 言語固有のカスタマイズ
