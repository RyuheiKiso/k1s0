# k1s0-schemaregistry ライブラリ設計

## 概要

Confluent Schema Registry クライアントライブラリ。`SchemaRegistryClient` トレイト（HTTP 実装: `HttpSchemaRegistryClient`）、`SchemaRegistryConfig`、`RegisteredSchema`、`SchemaType`（Avro/Json/Protobuf）を提供する。Kafka トピックのスキーマ登録・取得・互換性検証に使用する。

**配置先**: `regions/system/library/rust/schemaregistry/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SchemaRegistryClient` | トレイト | スキーマ登録・取得・互換性確認の抽象インターフェース |
| `HttpSchemaRegistryClient` | 構造体 | HTTP ベースの Schema Registry クライアント実装 |
| `MockSchemaRegistryClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `SchemaRegistryConfig` | 構造体 | Registry URL・認証情報・互換性モード設定 |
| `CompatibilityMode` | enum | スキーマ互換性モード（`Backward`・`Forward`・`Full`・`None`） |
| `RegisteredSchema` | 構造体 | 登録済みスキーマ（ID・バージョン・スキーマ文字列） |
| `SchemaType` | enum | スキーマ形式（`Avro`・`Json`・`Protobuf`） |
| `SchemaRegistryError` | enum | 登録・取得・互換性エラー型 |

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

**Cargo.toml への追加行**:

```toml
k1s0-schemaregistry = { path = "../../system/library/rust/schemaregistry" }
# テスト時にモックを有効化する場合:
k1s0-schemaregistry = { path = "../../system/library/rust/schemaregistry", features = ["mock"] }
```

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

**配置先**: `regions/system/library/go/schemaregistry/`

```
schemaregistry/
├── schemaregistry.go
├── config.go
├── client.go
├── schemaregistry_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/stretchr/testify v1.10.0`（`net/http` stdlib 使用）

**主要インターフェース**:

```go
type SchemaRegistryClient interface {
    RegisterSchema(ctx context.Context, subject, schema, schemaType string) (int, error)
    GetSchemaByID(ctx context.Context, id int) (*RegisteredSchema, error)
    GetLatestSchema(ctx context.Context, subject string) (*RegisteredSchema, error)
    ListSubjects(ctx context.Context) ([]string, error)
    CheckCompatibility(ctx context.Context, subject, schema string) (bool, error)
    HealthCheck(ctx context.Context) error
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/schemaregistry/`

```
schemaregistry/
├── package.json        # "@k1s0/schemaregistry", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # RegisteredSchema, SchemaRegistryClient, SchemaRegistryConfig, NotFoundError, SchemaRegistryError
└── __tests__/
    └── schemaregistry.test.ts
```

**主要 API**:

```typescript
export interface RegisteredSchema {
  id: number;
  subject: string;
  version: number;
  schema: string;
  schemaType: string;
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
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/schemaregistry/`

```
schemaregistry/
├── pubspec.yaml        # k1s0_schemaregistry, http: ^1.2.0
├── analysis_options.yaml
├── lib/
│   ├── schemaregistry.dart
│   └── src/
│       ├── types.dart      # RegisteredSchema, SchemaType
│       ├── config.dart     # SchemaRegistryConfig, subjectName
│       ├── client.dart     # SchemaRegistryClient abstract, HttpSchemaRegistryClient
│       └── error.dart      # NotFoundError, SchemaRegistryError
└── test/
    └── schemaregistry_test.dart
```

**カバレッジ目標**: 85%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — authlib ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ
