# system-api-registry-server 設計

> **ガイド**: 設計背景・実装例は [server.guide.md](./server.guide.md) を参照。

system tier の OpenAPI/Protobuf スキーマ集中管理サーバー。スキーマの登録・バージョン管理・破壊的変更検出・差分表示を提供する。Rust 実装。

## 概要

| 機能 | 説明 |
| --- | --- |
| スキーマ登録・バージョン管理 | OpenAPI 3.x / Protobuf スキーマの登録・バージョン履歴管理 |
| バリデーション | OpenAPI Validator / buf lint による登録時スキーマ検証 |
| 破壊的変更検出 | フィールド削除・型変更・必須化等の後方互換性破壊を自動検出 |
| 差分表示 | バージョン間のスキーマ差分を構造化 JSON で取得 |
| スキーマ更新通知 | スキーマ登録・更新時に Kafka `k1s0.system.apiregistry.schema_updated.v1` を発行 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| スキーマ検証 | openapi-spec-validator（subprocess 呼び出し）/ buf lint（subprocess 呼び出し） |

### 配置パス

配置: `regions/system/server/rust/api-registry/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_APIREG_` とする。

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

登録済みスキーマ一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `schema_type` | string | No | - | スキーマ種別でフィルタ（openapi/protobuf） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `schemas[].name` | string | スキーマ名 |
| `schemas[].description` | string | スキーマの説明 |
| `schemas[].schema_type` | string | スキーマ種別（`openapi` / `protobuf`） |
| `schemas[].latest_version` | int | 最新バージョン番号 |
| `schemas[].version_count` | int | 登録バージョン数 |
| `schemas[].created_at` | string | 初回登録日時 |
| `schemas[].updated_at` | string | 最終更新日時 |
| `pagination` | object | ページネーション（total_count, page, page_size, has_next） |

#### POST /api/v1/schemas

スキーマを新規登録する。初回バージョン（version 1）が作成される。登録時にバリデーションを実行し、エラーがある場合は 422 を返す。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `name` | string | Yes | スキーマ名 |
| `description` | string | Yes | スキーマの説明 |
| `schema_type` | string | Yes | スキーマ種別（`openapi` / `protobuf`） |
| `content` | string | Yes | スキーマ本文（YAML/JSON/proto） |

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | スキーマ名 |
| `description` | string | スキーマの説明 |
| `schema_type` | string | スキーマ種別 |
| `version` | int | バージョン番号（1） |
| `content_hash` | string | コンテンツの SHA-256 ハッシュ |
| `created_at` | string | 登録日時 |

**エラーレスポンス（422）**: `SYS_APIREG_SCHEMA_INVALID`（details にバリデーションエラー一覧）

#### GET /api/v1/schemas/:name

指定スキーマの最新バージョンのコンテンツを取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | スキーマ名 |
| `description` | string | スキーマの説明 |
| `schema_type` | string | スキーマ種別 |
| `latest_version` | int | 最新バージョン番号 |
| `content` | string | 最新バージョンのスキーマ本文 |
| `content_hash` | string | コンテンツの SHA-256 ハッシュ |
| `created_at` | string | 初回登録日時 |
| `updated_at` | string | 最終更新日時 |

**エラーレスポンス（404）**: `SYS_APIREG_SCHEMA_NOT_FOUND`

#### GET /api/v1/schemas/:name/versions

指定スキーマの全バージョン一覧をページネーション付きで取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | スキーマ名 |
| `versions[].version` | int | バージョン番号 |
| `versions[].content_hash` | string | コンテンツハッシュ |
| `versions[].breaking_changes` | bool | 破壊的変更フラグ |
| `versions[].registered_by` | string | 登録者ユーザー ID |
| `versions[].created_at` | string | 登録日時 |
| `pagination` | object | ページネーション |

#### GET /api/v1/schemas/:name/versions/:version

指定バージョンのスキーマコンテンツを取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | スキーマ名 |
| `version` | int | バージョン番号 |
| `schema_type` | string | スキーマ種別 |
| `content` | string | スキーマ本文 |
| `content_hash` | string | コンテンツハッシュ |
| `breaking_changes` | bool | 破壊的変更フラグ |
| `registered_by` | string | 登録者 |
| `created_at` | string | 登録日時 |

**エラーレスポンス（404）**: `SYS_APIREG_VERSION_NOT_FOUND`

#### POST /api/v1/schemas/:name/versions

既存スキーマに新バージョンを登録する。登録前に互換性チェックを自動実行し、破壊的変更が検出された場合はフラグを立てる（登録はそのまま行う）。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `content` | string | Yes | スキーマ本文 |
| `registered_by` | string | Yes | 登録者ユーザー ID |

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | スキーマ名 |
| `version` | int | バージョン番号 |
| `content_hash` | string | コンテンツハッシュ |
| `breaking_changes` | bool | 破壊的変更フラグ |
| `breaking_change_details` | BreakingChange[] | 破壊的変更の詳細リスト |
| `registered_by` | string | 登録者 |
| `created_at` | string | 登録日時 |

**エラーレスポンス（422）**: `SYS_APIREG_SCHEMA_INVALID`

#### DELETE /api/v1/schemas/:name/versions/:version

指定バージョンを削除する。最新バージョン（バージョン数 = 1）は削除できない。

**レスポンス**: 204 No Content

**エラーレスポンス（409）**: `SYS_APIREG_CANNOT_DELETE_LATEST`

#### POST /api/v1/schemas/:name/compatibility

指定スキーマに対して入力コンテンツの互換性チェックのみを実行する（登録しない）。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `content` | string | Yes | 検証対象のスキーマ本文 |
| `base_version` | int | No | 比較対象バージョン（省略時は最新） |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | スキーマ名 |
| `base_version` | int | 比較対象バージョン |
| `compatible` | bool | 後方互換性フラグ |
| `breaking_changes` | BreakingChange[] | 破壊的変更のリスト |
| `non_breaking_changes` | ChangeDetail[] | 非破壊的変更のリスト |

#### GET /api/v1/schemas/:name/diff

2 つのバージョン間の差分を取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `from` | int | No | `latest - 1` | 比較元バージョン |
| `to` | int | No | `latest` | 比較先バージョン |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | スキーマ名 |
| `from_version` | int | 比較元バージョン |
| `to_version` | int | 比較先バージョン |
| `breaking_changes` | bool | 破壊的変更フラグ |
| `diff.added` | ChangeDetail[] | 追加された要素 |
| `diff.modified` | ChangeDetail[] | 変更された要素 |
| `diff.removed` | ChangeDetail[] | 削除された要素 |

**エラーレスポンス（400）**: `SYS_APIREG_VALIDATION_ERROR`

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

スキーマの新規登録・新バージョン登録・バージョン削除時に Kafka トピック `k1s0.system.apiregistry.schema_updated.v1` にメッセージを送信する。

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.apiregistry.schema_updated.v1` |
| acks | `all` |
| message.timeout.ms | `5000` |
| キー | スキーマ名（例: `k1s0-tenant-api`） |

**イベント種別**:
- `SCHEMA_VERSION_REGISTERED` -- 新バージョン登録
- `SCHEMA_VERSION_DELETED` -- バージョン削除

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

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

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-server.md](../auth/server.md) -- system tier サーバー一覧
- [system-library-schemaregistry.md](../../libraries/data/schemaregistry.md) -- Kafka Avro スキーマレジストリライブラリ（Kafka 向け、当サーバーは REST/gRPC 向け）
- [proto設計.md](../../architecture/api/proto設計.md) -- Protobuf スキーマ設計ガイドライン
- [gRPC設計.md](../../architecture/api/gRPC設計.md) -- gRPC 設計ガイドライン
