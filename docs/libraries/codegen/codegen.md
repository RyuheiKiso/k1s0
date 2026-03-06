# k1s0-codegen

Rustサーバーの雛形を一括生成するコード生成ライブラリ。

## 概要

tier2/tier3開発者が新しいRustサーバーを作成する際、`ScaffoldConfig` を構築して `generate()` を呼び出すだけで、10種以上のファイルを一括生成する。

## クレート情報

| 項目 | 値 |
|------|-----|
| パッケージ名 | `k1s0-codegen` |
| パス | `regions/system/library/rust/codegen/` |
| テンプレートエンジン | Tera (`include_str!` 埋め込み) |
| 冪等性 | ファイル単位（既存スキップ） |

## 公開API

```rust
pub use config::{ScaffoldConfig, Tier, ApiStyle, DatabaseType};
pub use error::CodegenError;
pub use generator::{generate, GenerateResult};
pub use path::build_output_path;
```

### ScaffoldConfig

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `name` | `String` | kebab-case サーバー名（例: `user-profile`） |
| `tier` | `Tier` | `System` / `Business` / `Service` |
| `api_style` | `ApiStyle` | `Rest` / `Grpc` / `Both` |
| `database` | `DatabaseType` | `Postgres` / `None` |
| `description` | `String` | README用の説明文 |

### generate(config, output_dir) -> Result<GenerateResult>

指定ディレクトリにサーバー雛形を生成する。既存ファイルはスキップ。

### build_output_path(base, tier, name) -> PathBuf

`{base}/regions/{tier}/server/rust/{name}/` を返す。

## 生成ファイル一覧

| ファイル | 条件 | 内容 |
|---------|------|------|
| `Cargo.toml` | 常に | パッケージ定義、依存関係 |
| `src/main.rs` | 常に | K1s0App + TelemetryConfig初期化 |
| `src/lib.rs` | 常に | モジュール宣言 |
| `src/error.rs` | 常に | ServiceError (NotFound/BadRequest/Internal) |
| `config/config.yaml` | 常に | アプリ/サーバー/observability設定 |
| `README.md` | 常に | プロジェクト説明 |
| `src/adapter/mod.rs` | 常に | handler モジュール |
| `src/adapter/handler/mod.rs` | 常に | axum Router スケルトン |
| `src/domain/mod.rs` | 常に | entity/repository/service モジュール |
| `src/domain/entity/mod.rs` | 常に | 空 |
| `src/domain/repository/mod.rs` | 常に | 空 |
| `src/domain/service/mod.rs` | 常に | 空 |
| `src/usecase/mod.rs` | 常に | 空 |
| `src/infrastructure/mod.rs` | 常に | config モジュール |
| `src/infrastructure/config.rs` | 常に | Config struct (serde) |
| `build.rs` | gRPC時 | tonic-build コード生成 |
| `src/proto/.gitkeep` | gRPC時 | 生成コード配置先 |
| `api/proto/.../{name}.proto` | gRPC時 | サービス定義スケルトン |
| `migrations/001_initial.up.sql` | DB時 | 初期スキーマ |
| `migrations/001_initial.down.sql` | DB時 | ロールバック |

## 使用例

```rust
use k1s0_codegen::{generate, build_output_path, ScaffoldConfig, Tier, ApiStyle, DatabaseType};
use std::path::Path;

let config = ScaffoldConfig {
    name: "user-profile".into(),
    tier: Tier::Business,
    api_style: ApiStyle::Both,
    database: DatabaseType::Postgres,
    description: "User profile management service".into(),
};

let output = build_output_path(Path::new("/repo"), config.tier, &config.name);
let result = generate(&config, &output).unwrap();

println!("Created {} files, skipped {}", result.created.len(), result.skipped.len());
```

## 設計判断

| 判断 | 理由 |
|------|------|
| Tera テンプレートエンジン | テンプレートエンジン仕様で規定済み |
| `include_str!` 埋め込み | 外部ファイル依存なし、バイナリ完結 |
| ファイル単位冪等性（既存スキップ） | 部分再生成に対応、安全性確保 |
| auth-server の main.rs パターンを簡素化 | 既存パターンとの整合性 |
| ServiceError 3パターン初期生成 | 最小限で拡張可能 |

## モジュール構成

| ファイル | 役割 |
|---------|------|
| `config.rs` | ScaffoldConfig, Tier, ApiStyle, DatabaseType + validate() |
| `error.rs` | CodegenError (thiserror) |
| `generator.rs` | generate() オーケストレーション + render_file() |
| `path.rs` | Tier別出力パス計算 |
| `naming.rs` | kebab→snake/pascal/camel変換 |
| `context.rs` | Tera Context構築 |
| `templates/mod.rs` | create_tera_engine() + include_str! |
| `proto_parser.rs` | `.proto` ファイルパーサー (Feature: `proto`) |
| `cargo_updater.rs` | Cargo.toml 編集 (Feature: `cargo-update`) |
| `validator.rs` | 生成結果バリデーション |
| `client_sdk/` | Client SDK 生成 (Feature: `client-sdk`) → [client-sdk-generator.md](./client-sdk-generator.md) 参照 |

## 追加 Feature

### proto -- .proto ファイルパーサー

`.proto` ファイルを解析し、サービス定義・メソッド・メッセージを構造化データとして抽出する。

```toml
k1s0-codegen = { path = "...", features = ["proto"] }
```

#### API

```rust
pub struct ProtoService {
    pub package: String,        // e.g., "k1s0.business.accounting.v1"
    pub service_name: String,   // e.g., "AccountingService"
    pub methods: Vec<ProtoMethod>,
    pub messages: Vec<ProtoMessage>,
}

pub struct ProtoMethod {
    pub name: String,           // e.g., "CreateAccount"
    pub input_type: String,     // e.g., "CreateAccountRequest"
    pub output_type: String,    // e.g., "CreateAccountResponse"
}

pub struct ProtoMessage {
    pub name: String,
    pub fields: Vec<ProtoField>,
}

pub struct ProtoField {
    pub name: String,
    pub field_type: String,     // e.g., "string", "int32"
    pub number: u32,
}

pub fn parse_proto(path: &Path) -> Result<ProtoService, CodegenError>;
pub fn parse_proto_content(content: &str) -> Result<ProtoService, CodegenError>;
```

#### 使用例

```rust
use k1s0_codegen::proto_parser::parse_proto;

let service = parse_proto(Path::new("api/proto/accounting.proto"))?;
println!("Service: {} ({} methods)", service.service_name, service.methods.len());
```

### cargo-update -- Cargo.toml 自動編集

`Cargo.toml` に依存関係や Feature を安全に追加する。既存エントリはスキップ（冪等）。`toml_edit` による書式保持編集。

```toml
k1s0-codegen = { path = "...", features = ["cargo-update"] }
```

#### API

```rust
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub path: Option<String>,
    pub features: Vec<String>,
    pub optional: bool,
}

/// 依存関係を追加。既存ならスキップし false を返す。
pub fn add_dependency(cargo_toml_path: &Path, dep: &Dependency) -> Result<bool, CodegenError>;

/// Feature を追加。既存ならスキップし false を返す。
pub fn add_feature(cargo_toml_path: &Path, feature_name: &str, deps: &[&str]) -> Result<bool, CodegenError>;
```

#### 使用例

```rust
use k1s0_codegen::cargo_updater::{add_dependency, add_feature, Dependency};

let dep = Dependency {
    name: "serde".into(),
    version: Some("1".into()),
    path: None,
    features: vec!["derive".into()],
    optional: false,
};
add_dependency(Path::new("Cargo.toml"), &dep)?;
add_feature(Path::new("Cargo.toml"), "full", &["dep:serde", "dep:tokio"])?;
```

### validator -- 生成結果バリデーション

生成されたプロジェクトディレクトリの構造を検証する。

```rust
pub struct ValidationResult {
    pub errors: Vec<String>,    // 必須ファイル欠如
    pub warnings: Vec<String>,  // 推奨ファイル欠如
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool;  // errors が空なら true
}

pub fn validate_generated(output_dir: &Path) -> ValidationResult;
```

検証項目:

| ファイル | 分類 |
|---------|------|
| `Cargo.toml` | 必須 (error) |
| `src/main.rs` | 必須 (error) |
| `src/lib.rs` | 必須 (error) |
| `src/error.rs` | 推奨 (warning) |
| `config/config.yaml` | 推奨 (warning) |
