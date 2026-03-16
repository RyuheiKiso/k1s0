# business-domain-master-server 設計

accounting 領域のドメインマスタデータハブ。型安全な固定スキーマで領域固有バリデーションとテナント別カスタマイズを提供する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| biz_auditor 以上 | domain-master/read |
| biz_operator 以上 | domain-master/write |
| biz_admin のみ | domain-master/admin |


| 機能 | 説明 |
| --- | --- |
| カテゴリ管理 | 勘定科目、部門コード等のマスタカテゴリを定義・管理 |
| マスタ項目 CRUD | カテゴリ配下のマスタ項目の作成・取得・更新・削除 |
| 自動バージョニング | 項目更新時に before/after 差分を自動記録 |
| テナント別カスタマイズ | 表示名・属性のオーバーライド、有効/無効制御 |
| マージビュー | ベース項目 + テナントカスタマイズを統合した項目一覧を提供 |
| 領域固有バリデーション | カテゴリ定義の validation_schema (JSONB) による項目属性バリデーション |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/business/accounting/server/rust/domain-master/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

| 種別 | パス |
| --- | --- |
| Proto 定義 | `api/proto/k1s0/business/accounting/domainmaster/v1/domain_master.proto` |
| DB マイグレーション | `regions/business/accounting/database/domain-master-db/migrations/` |
| React クライアント | `regions/business/accounting/client/react/domain-master/` |
| Flutter クライアント | `regions/business/accounting/client/flutter/domain_master/` |

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| 型安全な固定スキーマ | master_categories, master_items, master_item_versions, tenant_master_extensions の 4 テーブルで構成 |
| バリデーション | カテゴリ定義の `validation_schema` (JSONB) による項目属性バリデーション |
| テナントカスタマイズ | 表示名・属性のオーバーライド、有効/無効制御 |
| バージョニング | 項目更新時に before/after 差分を自動記録 |
| DB | PostgreSQL 17 の `domain_master` スキーマ |
| Kafka | プロデューサー（3 トピック: category_changed / item_changed / tenant_extension_changed） |
| 認証 | JWT による認可。`biz_operator` / `biz_admin` ロールが必要 |
| ポート | 8210（REST）/ 50061（gRPC） |

---

## アーキテクチャ全体図

### レイヤー構成

| レイヤー | 責務 | コンポーネント |
| --- | --- | --- |
| API Layer | REST / gRPC エンドポイント | Domain Master Server (axum + tonic) |
| Business Layer | カテゴリ・項目管理、バージョニング、テナントカスタマイズ | UseCase + Domain Service |
| Data Layer | 永続化・イベント配信 | PostgreSQL 17, Kafka |

---

## データベース設計

### ER 図

```
master_categories 1──* master_items 1──* master_item_versions
                                    1──* tenant_master_extensions
master_items (self FK: parent_item_id)
```

### スキーマ: `domain_master`

#### master_categories（カテゴリ定義）

マスタデータのカテゴリ（勘定科目、部門コード等）を定義する。

```sql
CREATE TABLE domain_master.master_categories (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code              VARCHAR(100) NOT NULL UNIQUE,
    display_name      VARCHAR(255) NOT NULL,
    description       TEXT,
    validation_schema JSONB,
    is_active         BOOLEAN NOT NULL DEFAULT true,
    sort_order        INTEGER NOT NULL DEFAULT 0,
    created_by        VARCHAR(255) NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_master_categories_active ON domain_master.master_categories(is_active);
```

#### master_items（マスタ項目）

カテゴリ配下の個々のマスタ項目。

```sql
CREATE TABLE domain_master.master_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id     UUID NOT NULL REFERENCES domain_master.master_categories(id) ON DELETE CASCADE,
    code            VARCHAR(100) NOT NULL,
    display_name    VARCHAR(255) NOT NULL,
    description     TEXT,
    attributes      JSONB,
    parent_item_id  UUID REFERENCES domain_master.master_items(id),
    effective_from  TIMESTAMPTZ,
    effective_until TIMESTAMPTZ,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_by      VARCHAR(255) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_items_category_code UNIQUE (category_id, code)
);

CREATE INDEX idx_master_items_category ON domain_master.master_items(category_id);
CREATE INDEX idx_master_items_parent ON domain_master.master_items(parent_item_id);
CREATE INDEX idx_master_items_active ON domain_master.master_items(is_active);
CREATE INDEX idx_master_items_effective ON domain_master.master_items(effective_from, effective_until);
```

#### master_item_versions（変更履歴）

項目更新時の before/after 差分を自動記録する。

```sql
CREATE TABLE domain_master.master_item_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id         UUID NOT NULL REFERENCES domain_master.master_items(id) ON DELETE CASCADE,
    version_number  INTEGER NOT NULL,
    before_data     JSONB,
    after_data      JSONB,
    changed_by      VARCHAR(255) NOT NULL,
    change_reason   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_item_versions_item ON domain_master.master_item_versions(item_id);
CREATE INDEX idx_item_versions_created ON domain_master.master_item_versions(created_at);
```

#### tenant_master_extensions（テナントカスタマイズ）

テナントごとのマスタ項目オーバーライド。

```sql
CREATE TABLE domain_master.tenant_master_extensions (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             VARCHAR(255) NOT NULL,
    item_id               UUID NOT NULL REFERENCES domain_master.master_items(id) ON DELETE CASCADE,
    display_name_override VARCHAR(255),
    attributes_override   JSONB,
    is_enabled            BOOLEAN NOT NULL DEFAULT true,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_tenant_item UNIQUE (tenant_id, item_id)
);

CREATE INDEX idx_tenant_extensions_tenant ON domain_master.tenant_master_extensions(tenant_id);
CREATE INDEX idx_tenant_extensions_item ON domain_master.tenant_master_extensions(item_id);
```

---

## バリデーション

### validation_schema による項目属性バリデーション

カテゴリ定義の `validation_schema` (JSONB) で、そのカテゴリに属する項目の `attributes` をバリデーションする。

**validation_schema の例（勘定科目カテゴリ）：**

```json
{
  "type": "object",
  "required": ["account_type", "tax_category"],
  "properties": {
    "account_type": {
      "type": "string",
      "enum": ["asset", "liability", "equity", "revenue", "expense"]
    },
    "tax_category": {
      "type": "string",
      "enum": ["taxable", "tax_exempt", "non_taxable"]
    },
    "is_control_account": {
      "type": "boolean"
    }
  }
}
```

### バリデーションフロー

```
1. マスタ項目の作成・更新リクエスト
   ↓
2. category_id からカテゴリ定義を取得
   ↓
3. validation_schema が存在する場合
   ├── attributes を JSON Schema でバリデーション
   ├── 成功 → 処理続行
   └── 失敗 → BIZ_DOMAINMASTER_VALIDATION_ERROR 返却
   ↓
4. validation_schema が null の場合 → バリデーションスキップ
```

---

## テナントカスタマイズ

### マージビューの仕組み

テナント別マージビューは、ベースのマスタ項目とテナントカスタマイズを統合して返却する。

```
1. GET /api/v1/tenants/{tenant_id}/categories/{code}/items
   ↓
2. カテゴリ配下の master_items を取得
   ↓
3. tenant_master_extensions で tenant_id に一致するレコードを LEFT JOIN
   ↓
4. マージロジック
   ├── display_name_override が non-null → 表示名をオーバーライド
   ├── attributes_override が non-null → attributes を JSONB マージ
   ├── is_enabled = false → 項目を除外
   └── is_enabled = true または拡張なし → そのまま返却
   ↓
5. マージ済み項目一覧を返却
```

---

## API 設計

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `BIZ_DOMAINMASTER_` とする。

#### カテゴリ管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/categories` | カテゴリ一覧取得 | `biz_auditor` 以上 |
| POST | `/api/v1/categories` | カテゴリ作成 | `biz_operator` 以上 |
| GET | `/api/v1/categories/{code}` | カテゴリ詳細取得 | `biz_auditor` 以上 |
| PUT | `/api/v1/categories/{code}` | カテゴリ更新 | `biz_operator` 以上 |
| DELETE | `/api/v1/categories/{code}` | カテゴリ削除 | `biz_admin` のみ |

#### マスタ項目管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/categories/{code}/items` | カテゴリ配下の項目一覧 | `biz_auditor` 以上 |
| POST | `/api/v1/categories/{code}/items` | 項目作成 | `biz_operator` 以上 |
| GET | `/api/v1/categories/{code}/items/{item_code}` | 項目詳細取得 | `biz_auditor` 以上 |
| PUT | `/api/v1/categories/{code}/items/{item_code}` | 項目更新 | `biz_operator` 以上 |
| DELETE | `/api/v1/categories/{code}/items/{item_code}` | 項目削除 | `biz_admin` のみ |

#### バージョン履歴

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/categories/{code}/items/{item_code}/versions` | 項目の変更履歴一覧 | `biz_auditor` 以上 |

#### テナントカスタマイズ

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/tenants/{tenant_id}/items/{item_id}` | テナント拡張取得 | `biz_auditor` 以上 |
| PUT | `/api/v1/tenants/{tenant_id}/items/{item_id}` | テナント拡張作成・更新 | `biz_operator` 以上 |
| DELETE | `/api/v1/tenants/{tenant_id}/items/{item_id}` | テナント拡張削除 | `biz_admin` のみ |
| GET | `/api/v1/tenants/{tenant_id}/categories/{code}/items` | テナント別マージビュー | `biz_auditor` 以上 |

#### ヘルスチェック

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### エラーコード

| コード | HTTP Status | Description |
| --- | --- | --- |
| `BIZ_DOMAINMASTER_CATEGORY_NOT_FOUND` | 404 | 指定されたカテゴリが見つからない |
| `BIZ_DOMAINMASTER_ITEM_NOT_FOUND` | 404 | 指定されたマスタ項目が見つからない |
| `BIZ_DOMAINMASTER_DUPLICATE_CATEGORY` | 409 | カテゴリコードが重複 |
| `BIZ_DOMAINMASTER_DUPLICATE_ITEM` | 409 | カテゴリ内で項目コードが重複 |
| `BIZ_DOMAINMASTER_VALIDATION_ERROR` | 400 | validation_schema による属性バリデーションに失敗 |
| `BIZ_DOMAINMASTER_TENANT_EXT_NOT_FOUND` | 404 | テナント拡張が見つからない |
| `BIZ_DOMAINMASTER_CIRCULAR_PARENT` | 400 | 親項目の循環参照を検出 |
| `BIZ_AUTH_PERMISSION_DENIED` | 403 | 操作権限がない |

### gRPC サービス定義

`api/proto/k1s0/business/accounting/domainmaster/v1/domain_master.proto`

```protobuf
syntax = "proto3";

package k1s0.business.accounting.domainmaster.v1;

option go_package = "github.com/k1s0-platform/business-proto-go/accounting/domainmaster/v1;domainmasterv1";

import "google/protobuf/struct.proto";
import "k1s0/system/common/v1/types.proto";

service DomainMasterService {
  // カテゴリ管理
  rpc CreateCategory(CreateCategoryRequest) returns (CreateCategoryResponse);
  rpc GetCategory(GetCategoryRequest) returns (GetCategoryResponse);
  rpc UpdateCategory(UpdateCategoryRequest) returns (UpdateCategoryResponse);
  rpc DeleteCategory(DeleteCategoryRequest) returns (DeleteCategoryResponse);
  rpc ListCategories(ListCategoriesRequest) returns (ListCategoriesResponse);

  // マスタ項目管理
  rpc CreateItem(CreateItemRequest) returns (CreateItemResponse);
  rpc GetItem(GetItemRequest) returns (GetItemResponse);
  rpc UpdateItem(UpdateItemRequest) returns (UpdateItemResponse);
  rpc DeleteItem(DeleteItemRequest) returns (DeleteItemResponse);
  rpc ListItems(ListItemsRequest) returns (ListItemsResponse);

  // バージョン履歴
  rpc ListItemVersions(ListItemVersionsRequest) returns (ListItemVersionsResponse);

  // テナントカスタマイズ
  rpc GetTenantExtension(GetTenantExtensionRequest) returns (GetTenantExtensionResponse);
  rpc UpsertTenantExtension(UpsertTenantExtensionRequest) returns (UpsertTenantExtensionResponse);
  rpc DeleteTenantExtension(DeleteTenantExtensionRequest) returns (DeleteTenantExtensionResponse);
  rpc ListTenantMergedItems(ListTenantMergedItemsRequest) returns (ListTenantMergedItemsResponse);
}

// カテゴリ
message CreateCategoryRequest {
  string code = 1;
  string display_name = 2;
  string description = 3;
  google.protobuf.Struct validation_schema = 4;
  int32 sort_order = 5;
}

message CreateCategoryResponse {
  Category category = 1;
}

message GetCategoryRequest {
  string code = 1;
}

message GetCategoryResponse {
  Category category = 1;
}

message UpdateCategoryRequest {
  string code = 1;
  google.protobuf.Struct data = 2;
}

message UpdateCategoryResponse {
  Category category = 1;
}

message DeleteCategoryRequest {
  string code = 1;
}

message DeleteCategoryResponse {
  bool success = 1;
}

message ListCategoriesRequest {
  bool active_only = 1;
  k1s0.system.common.v1.Pagination pagination = 2;
}

message ListCategoriesResponse {
  repeated Category categories = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message Category {
  string id = 1;
  string code = 2;
  string display_name = 3;
  string description = 4;
  string validation_schema_json = 5;
  bool is_active = 6;
  int32 sort_order = 7;
  string created_by = 8;
  k1s0.system.common.v1.Timestamp created_at = 9;
  k1s0.system.common.v1.Timestamp updated_at = 10;
}

// マスタ項目
message CreateItemRequest {
  string category_code = 1;
  string code = 2;
  string display_name = 3;
  string description = 4;
  google.protobuf.Struct attributes = 5;
  optional string parent_item_id = 6;
  optional string effective_from = 7;
  optional string effective_until = 8;
  int32 sort_order = 9;
}

message CreateItemResponse {
  Item item = 1;
}

message GetItemRequest {
  string category_code = 1;
  string item_code = 2;
}

message GetItemResponse {
  Item item = 1;
}

message UpdateItemRequest {
  string category_code = 1;
  string item_code = 2;
  google.protobuf.Struct data = 3;
  string change_reason = 4;
}

message UpdateItemResponse {
  Item item = 1;
}

message DeleteItemRequest {
  string category_code = 1;
  string item_code = 2;
}

message DeleteItemResponse {
  bool success = 1;
}

message ListItemsRequest {
  string category_code = 1;
  bool active_only = 2;
  k1s0.system.common.v1.Pagination pagination = 3;
}

message ListItemsResponse {
  repeated Item items = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message Item {
  string id = 1;
  string category_id = 2;
  string code = 3;
  string display_name = 4;
  string description = 5;
  string attributes_json = 6;
  optional string parent_item_id = 7;
  optional string effective_from = 8;
  optional string effective_until = 9;
  bool is_active = 10;
  int32 sort_order = 11;
  string created_by = 12;
  k1s0.system.common.v1.Timestamp created_at = 13;
  k1s0.system.common.v1.Timestamp updated_at = 14;
}

// バージョン履歴
message ListItemVersionsRequest {
  string category_code = 1;
  string item_code = 2;
  k1s0.system.common.v1.Pagination pagination = 3;
}

message ListItemVersionsResponse {
  repeated ItemVersion versions = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message ItemVersion {
  string id = 1;
  string item_id = 2;
  int32 version_number = 3;
  string before_data_json = 4;
  string after_data_json = 5;
  string changed_by = 6;
  string change_reason = 7;
  k1s0.system.common.v1.Timestamp created_at = 8;
}

// テナントカスタマイズ
message GetTenantExtensionRequest {
  string tenant_id = 1;
  string item_id = 2;
}

message GetTenantExtensionResponse {
  TenantExtension extension = 1;
}

message UpsertTenantExtensionRequest {
  string tenant_id = 1;
  string item_id = 2;
  optional string display_name_override = 3;
  google.protobuf.Struct attributes_override = 4;
  bool is_enabled = 5;
}

message UpsertTenantExtensionResponse {
  TenantExtension extension = 1;
}

message DeleteTenantExtensionRequest {
  string tenant_id = 1;
  string item_id = 2;
}

message DeleteTenantExtensionResponse {
  bool success = 1;
}

message ListTenantMergedItemsRequest {
  string tenant_id = 1;
  string category_code = 2;
  bool active_only = 3;
  k1s0.system.common.v1.Pagination pagination = 4;
}

message ListTenantMergedItemsResponse {
  repeated MergedItem items = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message TenantExtension {
  string id = 1;
  string tenant_id = 2;
  string item_id = 3;
  optional string display_name_override = 4;
  string attributes_override_json = 5;
  bool is_enabled = 6;
  k1s0.system.common.v1.Timestamp created_at = 7;
  k1s0.system.common.v1.Timestamp updated_at = 8;
}

message MergedItem {
  Item base_item = 1;
  optional TenantExtension extension = 2;
  string merged_display_name = 3;
  string merged_attributes_json = 4;
}
```

### Kafka イベント

#### トピック 1: `k1s0.business.accounting.domainmaster.category_changed.v1`

```json
{
  "event_id": "uuid",
  "event_type": "category.created",
  "category_code": "account_titles",
  "operation": "CREATE",
  "after": { "code": "account_titles", "display_name": "勘定科目" },
  "changed_by": "user-uuid",
  "trace_id": "trace-uuid",
  "timestamp": "2026-03-07T09:00:00Z"
}
```

#### トピック 2: `k1s0.business.accounting.domainmaster.item_changed.v1`

```json
{
  "event_id": "uuid",
  "event_type": "item.updated",
  "category_code": "account_titles",
  "item_code": "1100",
  "operation": "UPDATE",
  "before": { "display_name": "現金", "attributes": { "account_type": "asset" } },
  "after": { "display_name": "現金及び預金", "attributes": { "account_type": "asset" } },
  "changed_by": "user-uuid",
  "trace_id": "trace-uuid",
  "timestamp": "2026-03-07T09:00:00Z"
}
```

#### トピック 3: `k1s0.business.accounting.domainmaster.tenant_extension_changed.v1`

```json
{
  "event_id": "uuid",
  "event_type": "tenant_extension.upserted",
  "tenant_id": "tenant-001",
  "item_id": "item-uuid",
  "operation": "UPSERT",
  "after": { "display_name_override": "現金（本社）", "is_enabled": true },
  "changed_by": "user-uuid",
  "trace_id": "trace-uuid",
  "timestamp": "2026-03-07T09:00:00Z"
}
```

---

## バックエンド設計 (Rust)

### ディレクトリ構成

```
regions/business/accounting/server/rust/domain-master/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── master_category.rs
│   │   │   ├── master_item.rs
│   │   │   ├── master_item_version.rs
│   │   │   └── tenant_master_extension.rs
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── category_repository.rs
│   │   │   ├── item_repository.rs
│   │   │   ├── item_version_repository.rs
│   │   │   └── tenant_extension_repository.rs
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── category_service.rs
│   │       ├── item_service.rs
│   │       └── validation_service.rs
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── manage_categories.rs
│   │   ├── manage_items.rs
│   │   ├── get_item_versions.rs
│   │   └── manage_tenant_extensions.rs
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── category_handler.rs
│   │   │   ├── item_handler.rs
│   │   │   ├── version_handler.rs
│   │   │   ├── tenant_handler.rs
│   │   │   └── error.rs
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   └── tonic_service.rs
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── response.rs
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs
│   │       └── rbac.rs
│   └── infrastructure/
│       ├── mod.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── app_config.rs
│       ├── persistence/
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   ├── category_repo_impl.rs
│       │   ├── item_repo_impl.rs
│       │   ├── item_version_repo_impl.rs
│       │   └── tenant_extension_repo_impl.rs
│       └── messaging/
│           ├── mod.rs
│           └── kafka_producer.rs
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── build.rs
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
└── README.md
```

---

## 設定フィールド

| カテゴリ | フィールド | 説明 |
| --- | --- | --- |
| server | `port` / `grpc_port` / `environment` | REST 8210 / gRPC 50061 |
| database | `host` / `port` / `name` / `schema` / `max_connections` | PostgreSQL `domain_master` スキーマ |
| kafka | `brokers` / `topics` | 3 トピック（category_changed / item_changed / tenant_extension_changed） |
| auth | `jwks_url` / `issuer` / `audience` | JWT 認証設定 |

---

## デプロイ

[business-server-deploy.md](../_common/deploy.md) に従い Helm Chart でデプロイする。

| パラメータ | 値 |
| --- | --- |
| replicas | 2 |
| resources.requests.cpu / memory | 200m / 256Mi |
| resources.limits.cpu / memory | 500m / 512Mi |
| readinessProbe.path | /readyz |
| livenessProbe.path | /healthz |

### Docker Compose (開発環境)

```yaml
domain-master-server:
  build:
    context: ./regions/business/accounting
    dockerfile: server/rust/domain-master/Dockerfile
  ports:
    - "8210:8210"
    - "9061:50061"
  environment:
    - DATABASE_URL=postgresql://k1s0:k1s0@postgres:5432/k1s0
    - KAFKA_BROKERS=kafka:9092
    - AUTH_JWKS_URL=http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs
  depends_on:
    - postgres
    - kafka
    - keycloak
```

---

## セキュリティ

### RBAC

```
1. JWT から user_id + roles を取得
2. ロールに応じたアクセス制御
   - biz_auditor 以上 → 読み取り操作
   - biz_operator 以上 → 書き込み操作
   - biz_admin のみ → 削除操作
3. テナントカスタマイズは tenant_id ベースのスコープ制御
4. 操作が許可されていない場合は 403 Forbidden を返却
```

---

## Cargo.toml

```toml
[package]
name = "k1s0-domain-master-server"
version = "0.1.0"
edition = "2021"

[lib]
name = "k1s0_domain_master_server"
path = "src/lib.rs"

[[bin]]
name = "k1s0-domain-master-server"
path = "src/main.rs"

[dependencies]
# Web framework
axum = { version = "0.8", features = ["macros", "multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"
regex = "1"

# Logging / Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Observability (共通テンプレート準拠)
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }
tracing-opentelemetry = "0.28"
prometheus = "0.13"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Telemetry library
k1s0-telemetry = { path = "../../../../../system/library/rust/telemetry", features = ["full"] }

# Auth library
k1s0-auth = { path = "../../../../../system/library/rust/auth" }

# Server common (error codes, auth middleware, RBAC, gRPC auth)
k1s0-server-common = { path = "../../../../../system/library/rust/server-common", features = ["axum", "grpc-auth"] }

# OpenAPI
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

[build-dependencies]
tonic-build = "0.12"

[features]
db-tests = []

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
tower = { version = "0.5", features = ["util"] }
testcontainers = "0.23"
axum-test = "17"
```

---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
