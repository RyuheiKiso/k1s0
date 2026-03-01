# k1s0-schemaregistry ライブラリ設計

## 概要

Confluent Schema Registry クライアントライブラリ。`SchemaRegistryClient` トレイト（HTTP 実装: `HttpSchemaRegistryClient`）、`SchemaRegistryConfig`、`RegisteredSchema`、`SchemaType`（Avro/Json/Protobuf）を提供する。Kafka トピックのスキーマ登録・取得・互換性検証に使用する。

**配置先**: `regions/system/library/rust/schemaregistry/`

## 公開 API

最小共通 API（全 4 言語）:

| メソッド | 説明 |
|---------|------|
| `register_schema(subject, schema, schema_type)` | スキーマを登録し、スキーマ ID を返す |
| `get_schema_by_id(id)` | スキーマ ID でスキーマを取得 |
| `get_latest_schema(subject)` | サブジェクトの最新スキーマを取得 |
| `get_schema_version(subject, version)` | サブジェクトの特定バージョンのスキーマを取得 |
| `list_subjects()` | 全サブジェクト名を取得 |
| `check_compatibility(subject, schema)` | スキーマの互換性を確認（※ Rust のみ第 3 引数 `schema_type` が必要） |
| `health_check()` | Schema Registry への疎通確認 |

> **`check_compatibility` の言語差異**: Rust 実装では `check_compatibility(subject, schema, schema_type: SchemaType)` の 3 引数を取る（Confluent API がスキーマ型を必要とするため）。Go/TypeScript/Dart は 2 引数（`subject`, `schema`）のみ。

> **`subject_name` の言語差異**:
> - **Rust**: `SchemaRegistryConfig::subject_name(topic)` -- 静的メソッド、`{topic}-value` サフィックス固定（`key` 指定不可）
> - **Go**: `(c *SchemaRegistryConfig) SubjectName(topic, keyOrValue)` -- インスタンスメソッド、`key`/`value` を指定可能
> - **TypeScript**: `subjectName(topic, keyOrValue: 'key' | 'value')` -- フリー関数、`key`/`value` を指定可能
> - **Dart**: `SchemaRegistryConfig.subjectName(topic, keyOrValue)` -- 静的メソッド、`key`/`value` を指定可能

Rust 追加 API（Rust のみ）:

| メソッド | 説明 |
|---------|------|
| `list_versions(subject)` | サブジェクトの全バージョン番号を取得 |
| `delete_subject(subject)` | サブジェクトを削除し、削除バージョン番号のリストを返す |

Rust 公開型:

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SchemaRegistryClient` | トレイト | スキーマ登録・取得・互換性確認の抽象インターフェース |
| `HttpSchemaRegistryClient` | 構造体 | HTTP ベースの Schema Registry クライアント実装 |
| `MockSchemaRegistryClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `SchemaRegistryConfig` | 構造体 | Registry URL・互換性モード・タイムアウト設定（`url`, `compatibility`, `timeout_secs: u64`（デフォルト 30 秒））。※ Rust は `username`/`password` フィールドを持たない（他 3 言語は対応済み） |
| `CompatibilityMode` | enum | スキーマ互換性モード（7 variants: `Backward`・`BackwardTransitive`・`Forward`・`ForwardTransitive`・`Full`・`FullTransitive`・`None`） |
| `RegisteredSchema` | 構造体 | 登録済みスキーマ（ID・バージョン・スキーマ文字列） |
| `SchemaType` | enum | スキーマ形式（`Avro`・`Json`・`Protobuf`） |
| `SchemaRegistryError` | enum | 登録・取得・互換性エラー型（6 variants: `Http(reqwest::Error)`, `SchemaNotFound { subject: String, version: Option<i32> }`, `CompatibilityViolation { subject, reason }`, `InvalidSchema(String)`, `Serialization(serde_json::Error)`, `Unavailable(String)`） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-schemaregistry"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
reqwest = { version = "0.12", features = ["json"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-schemaregistry = { path = "../../system/library/rust/schemaregistry" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
schemaregistry/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # SchemaRegistryClient トレイト・HttpSchemaRegistryClient・MockSchemaRegistryClient
│   ├── config.rs       # SchemaRegistryConfig・CompatibilityMode・subject_name ヘルパー
│   ├── error.rs        # SchemaRegistryError
│   └── schema.rs       # RegisteredSchema・SchemaType
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_schemaregistry::{
    HttpSchemaRegistryClient, SchemaRegistryClient, SchemaRegistryConfig, SchemaType,
};

let config = SchemaRegistryConfig::new("http://schema-registry:8081");
let client = HttpSchemaRegistryClient::new(config)?;

let topic = "k1s0.system.auth.user-created.v1";
let subject = SchemaRegistryConfig::subject_name(topic); // "<topic>-value"

// Protobuf スキーマを登録
let schema_id = client
    .register_schema(
        &subject,
        r#"syntax = "proto3"; message UserCreated { string user_id = 1; }"#,
        SchemaType::Protobuf,
    )
    .await?;

// 既存スキーマを ID で取得
let registered = client.get_schema_by_id(schema_id).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/schemaregistry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/stretchr/testify v1.10.0`（`net/http` stdlib 使用）

**主要インターフェース**:

```go
type SchemaRegistryClient interface {
    RegisterSchema(ctx context.Context, subject, schema, schemaType string) (int, error)
    GetSchemaByID(ctx context.Context, id int) (*RegisteredSchema, error)
    GetLatestSchema(ctx context.Context, subject string) (*RegisteredSchema, error)
    GetSchemaVersion(ctx context.Context, subject string, version int) (*RegisteredSchema, error)
    ListSubjects(ctx context.Context) ([]string, error)
    CheckCompatibility(ctx context.Context, subject, schema string) (bool, error)
    HealthCheck(ctx context.Context) error
}
```

> Go 実装: `CheckCompatibility` はスキーマ型引数なし（schema のみ）。`list_versions`・`delete_subject` は Rust のみ。

**公開型**:

```go
// 接続設定
type SchemaRegistryConfig struct {
    URL      string // Schema Registry のベース URL
    Username string // 基本認証のユーザー名（省略可能）
    Password string // 基本認証のパスワード（省略可能）
}

// SubjectName はトピック名からサブジェクト名を生成する（インスタンスメソッド）。
// Confluent の命名規則: <topic>-value または <topic>-key
func (c *SchemaRegistryConfig) SubjectName(topic, keyOrValue string) string

// Validate は設定を検証する。
func (c *SchemaRegistryConfig) Validate() error
```

> Go の `SubjectName` はインスタンスメソッドだが、config の値は参照しない。他言語では静的メソッド/フリー関数として実装されている。

```go
// 登録済みスキーマ
type RegisteredSchema struct {
    ID         int    `json:"id"`
    Subject    string `json:"subject"`
    Version    int    `json:"version"`
    Schema     string `json:"schema"`
    SchemaType string `json:"schemaType"`
}
```

**コンストラクタ**:

```go
// NewClient は新しい SchemaRegistryClient を生成する。
func NewClient(config *SchemaRegistryConfig) (SchemaRegistryClient, error)

// NewClientWithHTTPClient はカスタム http.Client を使う SchemaRegistryClient を生成する（テスト用）。
func NewClientWithHTTPClient(config *SchemaRegistryConfig, httpClient *http.Client) (SchemaRegistryClient, error)
```

**エラー型**:

```go
// NotFoundError はスキーマが見つからない場合のエラー。
type NotFoundError struct {
    Resource string
}

// IsNotFound は err が NotFoundError かどうかを返す。
func IsNotFound(err error) bool

// SchemaRegistryError は Schema Registry API のエラー。
type SchemaRegistryError struct {
    StatusCode int
    Message    string
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/schemaregistry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
/** スキーマ形式。 */
export type SchemaType = 'AVRO' | 'JSON' | 'PROTOBUF';

export interface RegisteredSchema {
  id: number;
  subject: string;
  version: number;
  schema: string;
  schemaType: SchemaType | string;
}

export interface SchemaRegistryConfig {
  url: string;
  username?: string;
  password?: string;
}

export function subjectName(topic: string, keyOrValue: 'key' | 'value'): string;

export interface SchemaRegistryClient {
  registerSchema(subject: string, schema: string, schemaType: string): Promise<number>;
  getSchemaById(id: number): Promise<RegisteredSchema>;
  getLatestSchema(subject: string): Promise<RegisteredSchema>;
  getSchemaVersion(subject: string, version: number): Promise<RegisteredSchema>;
  listSubjects(): Promise<string[]>;
  checkCompatibility(subject: string, schema: string): Promise<boolean>;
  healthCheck(): Promise<void>;
}

export class NotFoundError extends Error {
  constructor(resource: string);
}

export function isNotFound(err: unknown): boolean;

export class SchemaRegistryError extends Error {
  statusCode: number;
  constructor(statusCode: number, message: string);
}

export function validateSchemaRegistryConfig(config: SchemaRegistryConfig): void;

export class HttpSchemaRegistryClient implements SchemaRegistryClient {
  constructor(config: SchemaRegistryConfig);
  // ... SchemaRegistryClient の全メソッドを実装
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/schemaregistry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```dart
abstract class SchemaRegistryClient {
  Future<int> registerSchema(String subject, String schema, String schemaType);
  Future<RegisteredSchema> getSchemaById(int id);
  Future<RegisteredSchema> getLatestSchema(String subject);
  Future<RegisteredSchema> getSchemaVersion(String subject, int version);
  Future<List<String>> listSubjects();
  Future<bool> checkCompatibility(String subject, String schema);
  Future<void> healthCheck();
}

class HttpSchemaRegistryClient implements SchemaRegistryClient {
  HttpSchemaRegistryClient(SchemaRegistryConfig config, {http.Client? httpClient});
  // ... 上記メソッドすべてを実装
}
```

**公開型**:

```dart
/// スキーマ形式。
enum SchemaType {
  avro, json, protobuf;
  String toJson();
  static SchemaType fromString(String value);
}

/// 登録済みスキーマ。
class RegisteredSchema {
  final int id;
  final String subject;
  final int version;
  final String schema;
  final String schemaType;
  factory RegisteredSchema.fromJson(Map<String, dynamic> json);
}

/// 接続設定。
class SchemaRegistryConfig {
  final String url;       // required
  final String? username; // 基本認証のユーザー名（省略可能）
  final String? password; // 基本認証のパスワード（省略可能）
  static String subjectName(String topic, String keyOrValue);
  void validate(); // URL 空チェック（失敗時 SchemaRegistryError(0, ...) を throw）
}

/// スキーマが見つからない場合のエラー。
class NotFoundError implements Exception {
  final String resource;
}

/// Schema Registry API のエラー。
class SchemaRegistryError implements Exception {
  final int statusCode;
  final String message;
}

/// err が NotFoundError かどうかを返す。
bool isNotFound(Object? err);
```

> Dart の `validate()` はバリデーションエラー時に `SchemaRegistryError(0, ...)` を throw する（ステータスコード 0 はバリデーションエラーを表す）。

**カバレッジ目標**: 85%以上

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) — config ライブラリ
- [system-library-telemetry設計](../observability/telemetry.md) — telemetry ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) — authlib ライブラリ
- [system-library-messaging設計](../messaging/messaging.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](../messaging/kafka.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](../messaging/outbox.md) — k1s0-outbox ライブラリ

---
