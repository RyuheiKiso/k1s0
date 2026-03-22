# business-project-master-server 設計

taskmanagement 領域のプロジェクトマスタデータハブ。プロジェクトタイプ・ステータス定義・テナント拡張を型安全な固定スキーマで管理し、タスク管理サービス群に共有マスタデータを提供する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| project_type:read | プロジェクトタイプ一覧取得・単体取得 |
| project_type:write | プロジェクトタイプ作成・更新 |
| project_type:admin | プロジェクトタイプ削除 |
| status_definition:read | ステータス定義一覧取得・単体取得 |
| status_definition:write | ステータス定義作成・更新 |
| tenant_extension:read | テナント拡張取得 |
| tenant_extension:write | テナント拡張作成・更新・削除 |

Tier: `Tier::Business`。JWKS ベースの JWT 認証と、`require_permission(Tier::Business, "project_type"|"status_definition"|"tenant_extension", action)` による権限チェックを行う。

| 機能 | 説明 |
| --- | --- |
| プロジェクトタイプ CRUD | プロジェクトの分類定義（開発・運用・マーケティング等）を作成・取得・更新・削除する |
| ステータス定義 CRUD | プロジェクトタイプごとのステータス定義を作成・取得・更新・削除する |
| ステータス定義バージョニング | ステータス定義更新時に before/after 差分を自動記録する |
| テナント拡張 CRUD | テナントごとのプロジェクトカスタマイズ（表示名・属性オーバーライド）を管理する |
| Kafka イベント配信 | project_type_changed・status_definition_changed イベントを Kafka に非同期配信する |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/business/taskmanagement/server/rust/project-master/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

| 種別 | パス |
| --- | --- |
| Proto 定義 | `api/proto/k1s0/business/taskmanagement/projectmaster/v1/project_master.proto` |
| DB マイグレーション | `regions/business/taskmanagement/database/project-master-db/migrations/` |
| React クライアント | `regions/business/taskmanagement/client/react/project-master/` |
| Flutter クライアント | `regions/business/taskmanagement/client/flutter/project_master/` |

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

> **4言語パリティ（m-17 対応）**: taskmanagement サービスは現在 **Rust のみ** 実装されている。
> Go / TypeScript / Dart の実装は現時点では存在しない（`template-only` 状態）。
> 将来的に 4 言語対応を検討する際は、本ドキュメントの API 定義（REST/gRPC）を各言語のテンプレートとして活用すること。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | **Rust のみ**（Go / TypeScript / Dart は未実装） |
| 型安全な固定スキーマ | project_types, status_definitions, status_definition_versions, tenant_project_extensions の 4 テーブルで構成 |
| バリデーション | プロジェクトタイプのステータス遷移ルール整合性検証 |
| テナントカスタマイズ | 表示名・属性のオーバーライド、有効/無効制御 |
| バージョニング | ステータス定義更新時に before/after 差分を自動記録 |
| 楽観的ロック | version フィールドによる同時更新競合検知 |
| DB | PostgreSQL 17 の `project_master` スキーマ |
| Kafka | プロデューサー（2 トピック: project_type_changed / status_definition_changed） |
| イベント配信 | Outbox pattern（同一トランザクションで outbox_events に書き込み） |
| 認証 | JWT による認可。各リソースのロール（project_type / status_definition / tenant_extension）が必要 |
| ポート | 8210（REST）/ 9210（gRPC） |

---

## アーキテクチャ全体図

### レイヤー構成

| レイヤー | 責務 | コンポーネント |
| --- | --- | --- |
| API Layer | REST / gRPC エンドポイント | Project Master Server (axum + tonic) |
| Business Layer | プロジェクトタイプ・ステータス定義管理、バージョニング、テナント拡張 | UseCase + Domain Service |
| Data Layer | 永続化・イベント配信 | PostgreSQL 17, Kafka |

---

## データベース設計

### ER 図

```
project_types 1──* status_definitions 1──* status_definition_versions
project_types 1──* tenant_project_extensions
```

### スキーマ: `project_master`

#### project_types（プロジェクトタイプ定義）

プロジェクトの分類（開発・運用・マーケティング等）を定義する。

```sql
CREATE TABLE project_master.project_types (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code         VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description  TEXT,
    is_active    BOOLEAN NOT NULL DEFAULT true,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    version      INTEGER NOT NULL DEFAULT 1,
    created_by   VARCHAR(255) NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_project_types_active ON project_master.project_types(is_active);
```

#### status_definitions（ステータス定義）

プロジェクトタイプごとのステータスを定義する。

```sql
CREATE TABLE project_master.status_definitions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_type_id UUID NOT NULL REFERENCES project_master.project_types(id) ON DELETE CASCADE,
    code            VARCHAR(100) NOT NULL,
    display_name    VARCHAR(255) NOT NULL,
    description     TEXT,
    color           VARCHAR(7),
    is_terminal     BOOLEAN NOT NULL DEFAULT false,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    version         INTEGER NOT NULL DEFAULT 1,
    tenant_id       VARCHAR(255),
    created_by      VARCHAR(255) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_status_definitions_type_code UNIQUE (project_type_id, code)
);

CREATE INDEX idx_status_definitions_project_type ON project_master.status_definitions(project_type_id);
CREATE INDEX idx_status_definitions_tenant ON project_master.status_definitions(tenant_id);
```

#### status_definition_versions（ステータス定義変更履歴）

ステータス定義更新時の before/after 差分を自動記録する。

```sql
CREATE TABLE project_master.status_definition_versions (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status_definition_id UUID NOT NULL REFERENCES project_master.status_definitions(id) ON DELETE CASCADE,
    version_number       INTEGER NOT NULL,
    before_data          JSONB,
    after_data           JSONB,
    changed_by           VARCHAR(255) NOT NULL,
    change_reason        TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_status_definition_versions_def_version UNIQUE (status_definition_id, version_number)
);

CREATE INDEX idx_status_definition_versions_def ON project_master.status_definition_versions(status_definition_id, created_at DESC);
```

#### tenant_project_extensions（テナントプロジェクト拡張）

テナントごとのプロジェクトタイプカスタマイズ。

```sql
CREATE TABLE project_master.tenant_project_extensions (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             VARCHAR(255) NOT NULL,
    project_type_id       UUID NOT NULL REFERENCES project_master.project_types(id) ON DELETE CASCADE,
    display_name_override VARCHAR(255),
    attributes_override   JSONB,
    is_enabled            BOOLEAN NOT NULL DEFAULT true,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_tenant_project_extensions_tenant_type UNIQUE (tenant_id, project_type_id)
);

CREATE INDEX idx_tenant_project_extensions_tenant ON project_master.tenant_project_extensions(tenant_id);
CREATE INDEX idx_tenant_project_extensions_type ON project_master.tenant_project_extensions(project_type_id);
```

---

## API 設計

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `BIZ_PROJECTMASTER_` とする。

#### プロジェクトタイプ管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/project-types` | プロジェクトタイプ一覧取得 | `project_type:read` |
| POST | `/api/v1/project-types` | プロジェクトタイプ作成 | `project_type:write` |
| GET | `/api/v1/project-types/{code}` | プロジェクトタイプ詳細取得 | `project_type:read` |
| PUT | `/api/v1/project-types/{code}` | プロジェクトタイプ更新 | `project_type:write` |
| DELETE | `/api/v1/project-types/{code}` | プロジェクトタイプ削除 | `project_type:admin` |

#### ステータス定義管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/project-types/{code}/statuses` | ステータス定義一覧取得 | `status_definition:read` |
| POST | `/api/v1/project-types/{code}/statuses` | ステータス定義作成 | `status_definition:write` |
| GET | `/api/v1/project-types/{code}/statuses/{status_code}` | ステータス定義詳細取得 | `status_definition:read` |
| PUT | `/api/v1/project-types/{code}/statuses/{status_code}` | ステータス定義更新 | `status_definition:write` |
| DELETE | `/api/v1/project-types/{code}/statuses/{status_code}` | ステータス定義削除 | `project_type:admin` |

#### ステータス定義バージョン履歴

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/project-types/{code}/statuses/{status_code}/versions` | バージョン履歴一覧 | `status_definition:read` |

#### テナント拡張管理

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/tenants/{tenant_id}/project-types/{code}` | テナント拡張取得 | `tenant_extension:read` |
| PUT | `/api/v1/tenants/{tenant_id}/project-types/{code}` | テナント拡張作成・更新 | `tenant_extension:write` |
| DELETE | `/api/v1/tenants/{tenant_id}/project-types/{code}` | テナント拡張削除 | `tenant_extension:write` |

#### ヘルスチェック

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### エラーコード

| コード | HTTP Status | Description |
| --- | --- | --- |
| `BIZ_PROJECTMASTER_PROJECT_TYPE_NOT_FOUND` | 404 | 指定されたプロジェクトタイプが見つからない |
| `BIZ_PROJECTMASTER_STATUS_DEFINITION_NOT_FOUND` | 404 | 指定されたステータス定義が見つからない |
| `BIZ_PROJECTMASTER_DUPLICATE_PROJECT_TYPE` | 409 | プロジェクトタイプコードが重複 |
| `BIZ_PROJECTMASTER_DUPLICATE_STATUS` | 409 | プロジェクトタイプ内でステータスコードが重複 |
| `BIZ_PROJECTMASTER_TENANT_EXT_NOT_FOUND` | 404 | テナント拡張が見つからない |
| `BIZ_PROJECTMASTER_VERSION_CONFLICT` | 409 | 楽観的ロックによるバージョン競合 |
| `BIZ_AUTH_PERMISSION_DENIED` | 403 | 操作権限がない |

### gRPC サービス定義

`api/proto/k1s0/business/taskmanagement/projectmaster/v1/project_master.proto`

```protobuf
syntax = "proto3";

package k1s0.business.taskmanagement.projectmaster.v1;

option go_package = "github.com/k1s0-platform/business-proto-go/taskmanagement/projectmaster/v1;projectmasterv1";

import "google/protobuf/struct.proto";
import "k1s0/system/common/v1/types.proto";

service ProjectMasterService {
  // プロジェクトタイプ管理
  rpc CreateProjectType(CreateProjectTypeRequest) returns (CreateProjectTypeResponse);
  rpc GetProjectType(GetProjectTypeRequest) returns (GetProjectTypeResponse);
  rpc UpdateProjectType(UpdateProjectTypeRequest) returns (UpdateProjectTypeResponse);
  rpc DeleteProjectType(DeleteProjectTypeRequest) returns (DeleteProjectTypeResponse);
  rpc ListProjectTypes(ListProjectTypesRequest) returns (ListProjectTypesResponse);

  // ステータス定義管理
  rpc CreateStatusDefinition(CreateStatusDefinitionRequest) returns (CreateStatusDefinitionResponse);
  rpc GetStatusDefinition(GetStatusDefinitionRequest) returns (GetStatusDefinitionResponse);
  rpc UpdateStatusDefinition(UpdateStatusDefinitionRequest) returns (UpdateStatusDefinitionResponse);
  rpc DeleteStatusDefinition(DeleteStatusDefinitionRequest) returns (DeleteStatusDefinitionResponse);
  rpc ListStatusDefinitions(ListStatusDefinitionsRequest) returns (ListStatusDefinitionsResponse);

  // バージョン履歴
  rpc ListStatusDefinitionVersions(ListStatusDefinitionVersionsRequest) returns (ListStatusDefinitionVersionsResponse);

  // テナント拡張管理
  rpc GetTenantExtension(GetTenantExtensionRequest) returns (GetTenantExtensionResponse);
  rpc UpsertTenantExtension(UpsertTenantExtensionRequest) returns (UpsertTenantExtensionResponse);
  rpc DeleteTenantExtension(DeleteTenantExtensionRequest) returns (DeleteTenantExtensionResponse);
}
```

### Kafka イベント

#### トピック 1: `k1s0.business.taskmanagement.projectmaster.project_type_changed.v1`

```json
{
  "event_id": "uuid",
  "event_type": "project_type.created",
  "project_type_code": "development",
  "operation": "CREATE",
  "after": { "code": "development", "display_name": "開発プロジェクト" },
  "changed_by": "user-uuid",
  "trace_id": "trace-uuid",
  "timestamp": "2026-03-22T09:00:00Z"
}
```

#### トピック 2: `k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1`

```json
{
  "event_id": "uuid",
  "event_type": "status_definition.updated",
  "project_type_code": "development",
  "status_code": "in_progress",
  "operation": "UPDATE",
  "before": { "display_name": "進行中", "color": "#0000FF" },
  "after": { "display_name": "対応中", "color": "#0066CC" },
  "changed_by": "user-uuid",
  "trace_id": "trace-uuid",
  "timestamp": "2026-03-22T09:00:00Z"
}
```

---

## バックエンド設計 (Rust)

### ディレクトリ構成

```
regions/business/taskmanagement/server/rust/project-master/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── project_type.rs
│   │   │   ├── status_definition.rs
│   │   │   ├── status_definition_version.rs
│   │   │   └── tenant_project_extension.rs
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── project_type_repository.rs
│   │   │   ├── status_definition_repository.rs
│   │   │   ├── status_definition_version_repository.rs
│   │   │   └── tenant_extension_repository.rs
│   │   └── service/
│   │       ├── mod.rs
│   │       └── project_type_service.rs
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── manage_project_types.rs
│   │   ├── manage_status_definitions.rs
│   │   ├── get_status_definition_versions.rs
│   │   └── manage_tenant_extensions.rs
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── project_type_handler.rs
│   │   │   ├── status_definition_handler.rs
│   │   │   ├── tenant_extension_handler.rs
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
│       │   ├── project_type_repo_impl.rs
│       │   ├── status_definition_repo_impl.rs
│       │   ├── status_definition_version_repo_impl.rs
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
| server | `port` / `grpc_port` / `environment` | REST 8210 / gRPC 9210 |
| database | `host` / `port` / `name` / `schema` / `max_connections` | PostgreSQL `project_master` スキーマ |
| kafka | `brokers` / `topics` | 2 トピック（project_type_changed / status_definition_changed） |
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
project-master-server:
  build:
    context: ./regions/business/taskmanagement
    dockerfile: server/rust/project-master/Dockerfile
  profiles:
    - business
  ports:
    - "8210:8210"
    - "9210:9210"
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
2. リソース種別に応じたアクセス制御
   - project_type:read → プロジェクトタイプ読み取り操作
   - project_type:write → プロジェクトタイプ書き込み操作
   - project_type:admin → プロジェクトタイプ削除操作
   - status_definition:read/write → ステータス定義操作
   - tenant_extension:read/write → テナント拡張操作
3. 操作が許可されていない場合は 403 Forbidden を返却
```

---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
