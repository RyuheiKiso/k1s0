# system-api-registry-server 設計

system tier の OpenAPI/Protobuf スキーマ集中管理サーバー設計を定義する。REST/gRPC スキーマの登録・バージョン管理・破壊的変更検出・差分表示を提供する。スキーマ更新時は Kafka トピック `k1s0.system.apiregistry.schema_updated.v1` で通知する。Rust での実装を定義する。

## 概要

system tier の API スキーマレジストリサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| スキーマ登録・バージョン管理 | OpenAPI 3.x / Protobuf スキーマの登録・バージョン履歴管理 |
| バリデーション | OpenAPI Validator / buf lint による登録時スキーマ検証 |
| 破壊的変更検出 | フィールド削除・型変更・必須化等の後方互換性破壊を自動検出 |
| 差分表示 | バージョン間のスキーマ差分を構造化 JSON で取得 |
| スキーマ更新通知 | スキーマ登録・更新時に Kafka `k1s0.system.apiregistry.schema_updated.v1` を発行 |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC | tonic v0.12 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| スキーマ検証 | openapi-spec-validator（subprocess 呼び出し）/ buf lint（subprocess 呼び出し） |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/api-registry/` |

---

## 設計方針

[認証認可設計.md](../../auth/design/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| スキーマ種別 | `openapi`（OpenAPI 3.x YAML/JSON）と `protobuf`（.proto ファイル）の 2 種類をサポート |
| バリデーション | 登録時に subprocess 経由で openapi-spec-validator（OpenAPI）または buf lint（Protobuf）を実行し検証エラーを返す |
| 破壊的変更検出 | 新バージョン登録時に前バージョンとの比較を行い、フィールド削除・型変更・必須化・パス削除等の変更を検出する |
| 差分表示 | バージョン間の差分を `added` / `modified` / `removed` に分類した構造化 JSON で提供する |
| kafka-schemaregistry との対比 | kafka-schemaregistry ライブラリは Kafka Avro スキーマ向け。当サーバーは REST/gRPC スキーマのレジストリとして機能する |
| DB | PostgreSQL の `apiregistry` スキーマ（api_schemas, api_schema_versions テーブル） |
| Kafka | プロデューサー（`k1s0.system.apiregistry.schema_updated.v1`） |
| 認証 | JWTによる認可。管理系エンドポイントは `sys_operator` / `sys_admin` ロールが必要 |
| ポート | 8101（REST）/ 9090（gRPC） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../api/gateway/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_APIREG_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/schemas` | スキーマ一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/schemas` | スキーマ登録（初回バージョン） | `sys_operator` 以上 |
| GET | `/api/v1/schemas/:name` | スキーマ取得（最新バージョン） | `sys_auditor` 以上 |
| GET | `/api/v1/schemas/:name/versions` | バージョン一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/schemas/:name/versions/:version` | 特定バージョン取得 | `sys_auditor` 以上 |
| POST | `/api/v1/schemas/:name/versions` | 新バージョン登録 | `sys_operator` 以上 |
| DELETE | `/api/v1/schemas/:name/versions/:version` | バージョン削除 | `sys_admin` のみ |
| POST | `/api/v1/schemas/:name/compatibility` | 互換性チェック（破壊的変更検出） | `sys_operator` 以上 |
| GET | `/api/v1/schemas/:name/diff` | バージョン間差分取得 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/schemas

登録済みスキーマ一覧をページネーション付きで取得する。`schema_type` でフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `schema_type` | string | No | - | スキーマ種別でフィルタ（openapi/protobuf） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "schemas": [
    {
      "name": "k1s0-tenant-api",
      "description": "テナント管理 API スキーマ",
      "schema_type": "openapi",
      "latest_version": 3,
      "version_count": 3,
      "created_at": "2026-02-10T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    },
    {
      "name": "k1s0-notification-proto",
      "description": "通知サービス Protobuf スキーマ",
      "schema_type": "protobuf",
      "latest_version": 1,
      "version_count": 1,
      "created_at": "2026-02-15T10:00:00.000+00:00",
      "updated_at": "2026-02-15T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 12,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

#### POST /api/v1/schemas

スキーマを新規登録する。初回バージョン（version 1）が作成される。登録時にバリデーション（openapi-spec-validator または buf lint）を実行し、エラーがある場合は 422 を返す。

**リクエスト（OpenAPI）**

```json
{
  "name": "k1s0-tenant-api",
  "description": "テナント管理 API スキーマ",
  "schema_type": "openapi",
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 1.0.0\npaths:\n  /api/v1/tenants:\n    get:\n      summary: テナント一覧\n      responses:\n        '200':\n          description: OK\n"
}
```

**リクエスト（Protobuf）**

```json
{
  "name": "k1s0-notification-proto",
  "description": "通知サービス Protobuf スキーマ",
  "schema_type": "protobuf",
  "content": "syntax = \"proto3\";\npackage k1s0.system.notification.v1;\n\nservice NotificationService {\n  rpc SendNotification(SendNotificationRequest) returns (SendNotificationResponse);\n}\n\nmessage SendNotificationRequest {\n  string channel_id = 1;\n  string recipient = 2;\n}\n\nmessage SendNotificationResponse {\n  string notification_id = 1;\n  string status = 2;\n}\n"
}
```

**レスポンス（201 Created）**

```json
{
  "name": "k1s0-tenant-api",
  "description": "テナント管理 API スキーマ",
  "schema_type": "openapi",
  "version": 1,
  "content_hash": "sha256:a1b2c3d4e5f6...",
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（422 Unprocessable Entity）**

```json
{
  "error": {
    "code": "SYS_APIREG_SCHEMA_INVALID",
    "message": "schema validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "content", "message": "[line 5] info.version is required"},
      {"field": "content", "message": "[line 12] response schema must have a type"}
    ]
  }
}
```

#### GET /api/v1/schemas/:name

指定スキーマの最新バージョンのコンテンツと全バージョンの概要を取得する。

**レスポンス（200 OK）**

```json
{
  "name": "k1s0-tenant-api",
  "description": "テナント管理 API スキーマ",
  "schema_type": "openapi",
  "latest_version": 3,
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 3.0.0\n...",
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "created_at": "2026-02-10T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_APIREG_SCHEMA_NOT_FOUND",
    "message": "schema not found: k1s0-tenant-api",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/schemas/:name/versions

指定スキーマの全バージョン一覧をページネーション付きで取得する。

**レスポンス（200 OK）**

```json
{
  "name": "k1s0-tenant-api",
  "versions": [
    {
      "version": 3,
      "content_hash": "sha256:f6e5d4c3b2a1...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-20T12:30:00.000+00:00"
    },
    {
      "version": 2,
      "content_hash": "sha256:e5d4c3b2a1f6...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-15T10:00:00.000+00:00"
    },
    {
      "version": 1,
      "content_hash": "sha256:a1b2c3d4e5f6...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-10T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 3,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

#### GET /api/v1/schemas/:name/versions/:version

指定バージョンのスキーマコンテンツを取得する。

**レスポンス（200 OK）**

```json
{
  "name": "k1s0-tenant-api",
  "version": 2,
  "schema_type": "openapi",
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 2.0.0\n...",
  "content_hash": "sha256:e5d4c3b2a1f6...",
  "breaking_changes": false,
  "registered_by": "user-001",
  "created_at": "2026-02-15T10:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_APIREG_VERSION_NOT_FOUND",
    "message": "schema version not found: k1s0-tenant-api@2",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/schemas/:name/versions

既存スキーマに新バージョンを登録する。登録前に互換性チェックを自動実行し、破壊的変更が検出された場合はフラグを立てる（登録はそのまま行う）。

**リクエスト**

```json
{
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 3.0.0\npaths:\n  /api/v1/tenants:\n    get:\n      summary: テナント一覧\n      ...\n",
  "registered_by": "user-001"
}
```

**レスポンス（201 Created）**

```json
{
  "name": "k1s0-tenant-api",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": false,
  "breaking_change_details": [],
  "registered_by": "user-001",
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（201 Created -- 破壊的変更あり）**

```json
{
  "name": "k1s0-tenant-api",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": true,
  "breaking_change_details": [
    {
      "change_type": "field_removed",
      "path": "/api/v1/tenants GET response.properties.legacy_id",
      "description": "フィールド 'legacy_id' が削除されました"
    },
    {
      "change_type": "type_changed",
      "path": "/api/v1/tenants/{id} GET response.properties.created_at",
      "description": "'created_at' の型が string から integer に変更されました"
    }
  ],
  "registered_by": "user-001",
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（422 Unprocessable Entity）**

```json
{
  "error": {
    "code": "SYS_APIREG_SCHEMA_INVALID",
    "message": "schema validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "content", "message": "[line 8] path must start with '/'"}
    ]
  }
}
```

#### DELETE /api/v1/schemas/:name/versions/:version

指定バージョンを削除する。最新バージョン（バージョン数 = 1）は削除できない。

**レスポンス（204 No Content）**

レスポンスボディなし。

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_APIREG_CANNOT_DELETE_LATEST",
    "message": "cannot delete the only remaining version of schema: k1s0-tenant-api",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/schemas/:name/compatibility

指定スキーマに対して入力コンテンツの互換性チェックのみを実行する（登録しない）。`base_version` を省略した場合は最新バージョンと比較する。

**リクエスト**

```json
{
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 4.0.0\n...",
  "base_version": 3
}
```

**レスポンス（200 OK）**

```json
{
  "name": "k1s0-tenant-api",
  "base_version": 3,
  "compatible": false,
  "breaking_changes": [
    {
      "change_type": "field_removed",
      "path": "/api/v1/tenants GET response.properties.name",
      "description": "フィールド 'name' が削除されました"
    }
  ],
  "non_breaking_changes": [
    {
      "change_type": "field_added",
      "path": "/api/v1/tenants GET response.properties.display_name",
      "description": "フィールド 'display_name' が追加されました"
    }
  ]
}
```

#### GET /api/v1/schemas/:name/diff

2 つのバージョン間の差分を取得する。`from` と `to` クエリパラメータでバージョンを指定する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `from` | int | No | `latest - 1` | 比較元バージョン |
| `to` | int | No | `latest` | 比較先バージョン |

**レスポンス（200 OK）**

```json
{
  "name": "k1s0-tenant-api",
  "from_version": 2,
  "to_version": 3,
  "breaking_changes": false,
  "diff": {
    "added": [
      {
        "path": "/api/v1/tenants GET response.properties.display_name",
        "type": "object",
        "description": "新フィールド: display_name（表示名）"
      }
    ],
    "modified": [
      {
        "path": "/api/v1/tenants GET summary",
        "before": "テナント一覧",
        "after": "テナント一覧取得"
      }
    ],
    "removed": []
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_APIREG_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "from", "message": "'from' version must be less than 'to' version"}
    ]
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_APIREG_SCHEMA_NOT_FOUND` | 404 | 指定されたスキーマが見つからない |
| `SYS_APIREG_VERSION_NOT_FOUND` | 404 | 指定されたバージョンが見つからない |
| `SYS_APIREG_ALREADY_EXISTS` | 409 | 同一名のスキーマが既に存在する |
| `SYS_APIREG_CANNOT_DELETE_LATEST` | 409 | 唯一の残存バージョンは削除できない |
| `SYS_APIREG_SCHEMA_INVALID` | 422 | スキーマのバリデーションエラー（openapi-spec-validator / buf lint） |
| `SYS_APIREG_VALIDATION_ERROR` | 400 | リクエストパラメータのバリデーションエラー |
| `SYS_APIREG_VALIDATOR_ERROR` | 502 | 外部バリデーター（openapi-spec-validator / buf）の実行エラー |
| `SYS_APIREG_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

proto ファイルは `api/proto/k1s0/system/apiregistry/v1/api_registry.proto` に配置する。

```protobuf
syntax = "proto3";
package k1s0.system.apiregistry.v1;

service ApiRegistryService {
  rpc GetSchema(GetSchemaRequest) returns (GetSchemaResponse);
  rpc GetSchemaVersion(GetSchemaVersionRequest) returns (GetSchemaVersionResponse);
  rpc CheckCompatibility(CheckCompatibilityRequest) returns (CheckCompatibilityResponse);
}

message GetSchemaRequest {
  string name = 1;
}

message GetSchemaResponse {
  ApiSchemaProto schema = 1;
  string latest_content = 2;
}

message GetSchemaVersionRequest {
  string name = 1;
  uint32 version = 2;
}

message GetSchemaVersionResponse {
  ApiSchemaVersionProto version = 1;
}

message CheckCompatibilityRequest {
  string name = 1;
  string content = 2;
  optional uint32 base_version = 3;
}

message CheckCompatibilityResponse {
  string name = 1;
  uint32 base_version = 2;
  CompatibilityResultProto result = 3;
}

message ApiSchemaProto {
  string name = 1;
  string description = 2;
  string schema_type = 3;
  uint32 latest_version = 4;
  uint32 version_count = 5;
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp updated_at = 7;
}

message ApiSchemaVersionProto {
  string name = 1;
  uint32 version = 2;
  string schema_type = 3;
  string content = 4;
  string content_hash = 5;
  bool breaking_changes = 6;
  string registered_by = 7;
  google.protobuf.Timestamp created_at = 8;
}

message CompatibilityResultProto {
  bool compatible = 1;
  repeated string breaking_changes = 2;
  repeated string non_breaking_changes = 3;
}
```

---

## Kafka メッセージング設計

### スキーマ更新通知

スキーマの新規登録・新バージョン登録・バージョン削除時に以下のメッセージを Kafka トピック `k1s0.system.apiregistry.schema_updated.v1` に送信する。

**メッセージフォーマット（新バージョン登録）**

```json
{
  "event_type": "SCHEMA_VERSION_REGISTERED",
  "schema_name": "k1s0-tenant-api",
  "schema_type": "openapi",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": false,
  "registered_by": "user-001",
  "timestamp": "2026-02-20T12:30:00.000+00:00"
}
```

**メッセージフォーマット（バージョン削除）**

```json
{
  "event_type": "SCHEMA_VERSION_DELETED",
  "schema_name": "k1s0-tenant-api",
  "schema_type": "openapi",
  "version": 1,
  "deleted_by": "user-001",
  "timestamp": "2026-02-20T15:00:00.000+00:00"
}
```

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.apiregistry.schema_updated.v1` |
| acks | `all` |
| message.timeout.ms | `5000` |
| キー | スキーマ名（例: `k1s0-tenant-api`） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース・ドメインサービス）
  ^
usecase（ビジネスロジック）
  ^
adapter（REST ハンドラー・gRPC ハンドラー）
  ^
infrastructure（DB接続・Kafka Producer・バリデーター subprocess・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `ApiSchema`, `ApiSchemaVersion`, `CompatibilityResult`, `SchemaDiff` | エンティティ定義 |
| domain/repository | `ApiSchemaRepository`, `ApiSchemaVersionRepository` | リポジトリトレイト |
| domain/service | `ApiRegistryDomainService` | 破壊的変更検出ロジック・差分算出・コンテンツハッシュ計算 |
| usecase | `ListSchemasUsecase`, `RegisterSchemaUsecase`, `GetSchemaUsecase`, `ListVersionsUsecase`, `GetSchemaVersionUsecase`, `RegisterVersionUsecase`, `DeleteVersionUsecase`, `CheckCompatibilityUsecase`, `GetDiffUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `ApiSchemaPostgresRepository`, `ApiSchemaVersionPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/validator | `OpenApiValidator`, `ProtobufValidator` | subprocess 経由バリデーター実装 |
| infrastructure/messaging | `SchemaUpdatedKafkaProducer` | Kafka プロデューサー（スキーマ更新通知） |

### ドメインモデル

#### ApiSchema

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | String | スキーマ名（例: `k1s0-tenant-api`） |
| `description` | String | スキーマの説明 |
| `schema_type` | String | スキーマ種別（`openapi` / `protobuf`） |
| `latest_version` | u32 | 最新バージョン番号 |
| `version_count` | u32 | 登録バージョン数 |
| `created_at` | DateTime\<Utc\> | 初回登録日時 |
| `updated_at` | DateTime\<Utc\> | 最終更新日時 |

#### ApiSchemaVersion

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | String | スキーマ名 |
| `version` | u32 | バージョン番号（1 始まりの連番） |
| `schema_type` | String | スキーマ種別 |
| `content` | String | スキーマ本文（YAML/JSON/proto） |
| `content_hash` | String | コンテンツの SHA-256 ハッシュ（重複検出に使用） |
| `breaking_changes` | bool | 前バージョンからの破壊的変更フラグ |
| `breaking_change_details` | Vec\<BreakingChange\> | 破壊的変更の詳細リスト |
| `registered_by` | String | 登録者のユーザー ID |
| `created_at` | DateTime\<Utc\> | 登録日時 |

#### CompatibilityResult

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `compatible` | bool | 後方互換性フラグ（破壊的変更なし = true） |
| `breaking_changes` | Vec\<BreakingChange\> | 破壊的変更のリスト |
| `non_breaking_changes` | Vec\<ChangeDetail\> | 非破壊的変更のリスト |

#### BreakingChange（破壊的変更の種別）

| change_type | 説明 |
| --- | --- |
| `field_removed` | レスポンス/リクエストフィールドの削除 |
| `type_changed` | フィールドの型変更 |
| `required_added` | オプションフィールドの必須化 |
| `path_removed` | API パスの削除 |
| `method_removed` | HTTPメソッドの削除（GET/POST等） |
| `enum_value_removed` | enum 値の削除 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (apiregistry_handler.rs)    │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_schemas / register_schema          │   │
                    │  │  get_schema / list_versions              │   │
                    │  │  get_schema_version                      │   │
                    │  │  register_version / delete_version       │   │
                    │  │  check_compatibility / get_diff          │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (apiregistry_grpc.rs)       │   │
                    │  │  GetSchema / GetSchemaVersion            │   │
                    │  │  CheckCompatibility                      │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  ListSchemasUsecase / RegisterSchemaUsecase /   │
                    │  GetSchemaUsecase / ListVersionsUsecase /       │
                    │  GetSchemaVersionUsecase /                      │
                    │  RegisterVersionUsecase / DeleteVersionUsecase /│
                    │  CheckCompatibilityUsecase / GetDiffUsecase     │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  ApiSchema,     │              │ ApiSchemaRepository        │   │
    │  ApiSchemaVer,  │              │ ApiSchemaVersionRepository │   │
    │  Compatibility  │              │ (trait)                    │   │
    │  Result,        │              └──────────┬─────────────────┘   │
    │  SchemaDiff     │                         │                     │
    └────────────────┘                         │                     │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ ApiRegistry    │            │                     │
                 │ DomainService  │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ ApiSchemaPostgres       │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ ApiSchemaVersion       │  │
                    │  │ OpenApi      │  │ PostgresRepository     │  │
                    │  │ Validator    │  └────────────────────────┘  │
                    │  ├──────────────┤  ┌────────────────────────┐  │
                    │  │ Protobuf     │  │ Database               │  │
                    │  │ Validator    │  │ Config                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐                              │
                    │  │ Config       │                              │
                    │  │ Loader       │                              │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## DB スキーマ

PostgreSQL の `apiregistry` スキーマに以下のテーブルを配置する。

```sql
CREATE SCHEMA IF NOT EXISTS apiregistry;

CREATE TABLE apiregistry.api_schemas (
    name         TEXT PRIMARY KEY,
    description  TEXT NOT NULL DEFAULT '',
    schema_type  TEXT NOT NULL CHECK (schema_type IN ('openapi', 'protobuf')),
    latest_version INTEGER NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE apiregistry.api_schema_versions (
    name                    TEXT NOT NULL REFERENCES apiregistry.api_schemas(name) ON DELETE CASCADE,
    version                 INTEGER NOT NULL,
    content                 TEXT NOT NULL,
    content_hash            TEXT NOT NULL,
    breaking_changes        BOOLEAN NOT NULL DEFAULT false,
    breaking_change_details JSONB NOT NULL DEFAULT '[]',
    registered_by           TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (name, version)
);

CREATE INDEX idx_api_schema_versions_name ON apiregistry.api_schema_versions(name);
CREATE INDEX idx_api_schema_versions_content_hash ON apiregistry.api_schema_versions(content_hash);
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "api-registry"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic: "k1s0.system.apiregistry.schema_updated.v1"

validator:
  openapi_spec_validator_path: "/usr/local/bin/openapi-spec-validator"
  buf_path: "/usr/local/bin/buf"
  timeout_seconds: 30
```

---

## デプロイ

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。api-registry 固有の values は以下の通り。

```yaml
# values-api-registry.yaml（infra/helm/services/system/api-registry/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/api-registry
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 9090

service:
  type: ClusterIP
  port: 80
  grpcPort: 9090

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 4
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/api-registry/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/api-registry/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

---

## 関連ドキュメント

- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka メッセージング設計
- [API設計.md](../../api/gateway/API設計.md) -- REST API 設計ガイドライン
- [認証認可設計.md](../../auth/design/認証認可設計.md) -- RBAC 認可モデル
- [可観測性設計.md](../../observability/overview/可観測性設計.md) -- メトリクス・トレース設計
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- コーディング規約
- [system-library-概要.md](../../system-libraries/overview/概要.md) -- ライブラリ一覧
- [system-server設計.md](system-server設計.md) -- system tier サーバー一覧
- [tier-architecture.md](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [system-library-schemaregistry設計.md](../../system-libraries/data/schemaregistry設計.md) -- Kafka Avro スキーマレジストリライブラリ（Kafka 向け、当サーバーは REST/gRPC 向け）
- [proto設計.md](../../api/protocols/proto設計.md) -- Protobuf スキーマ設計ガイドライン
- [gRPC設計.md](../../api/protocols/gRPC設計.md) -- gRPC 設計ガイドライン
