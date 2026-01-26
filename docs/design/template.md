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
| `language` | 言語 | `rust`, `go`, `typescript`, `dart` |
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
//! {{ feature_name_pascal }} サービス
//!
//! {{ feature_name }} のエントリーポイント。

use k1s0_config::{ServiceArgs, ServiceInit};
use k1s0_observability::{ObservabilityConfig, LogEntry};

mod application;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 設定の読み込み
    let args = ServiceArgs::from_env();
    let init = ServiceInit::from_args(&args)?;

    // 観測性の初期化
    let obs_config = ObservabilityConfig::builder()
        .service_name("{{ feature_name }}")
        .env(init.env())
        .build()?;

    LogEntry::info("サービスを起動しています")
        .with_service(&obs_config);

    // TODO: サーバの起動

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

## managed/protected パス

### managed パス（CLI が自動更新）

| テンプレート | managed パス |
|-------------|-------------|
| backend-rust | `deploy/`, `buf.yaml`, `buf.gen.yaml` |
| backend-go | `deploy/`, `buf.yaml`, `buf.gen.yaml` |
| frontend-react | `deploy/` |
| frontend-flutter | `deploy/` |

### protected パス（CLI が変更しない）

| テンプレート | protected パス |
|-------------|---------------|
| backend-rust | `src/domain/`, `src/application/`, `README.md` |
| backend-go | `internal/domain/`, `internal/application/`, `README.md` |
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
