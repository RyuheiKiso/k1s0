# system-tenant-server 設計

system tier のテナント管理サーバー設計を定義する。マルチテナンシーを実現するためのテナントプロビジョニング・管理・分離制御を提供し、Keycloak realm と連携してテナント単位の認証境界を確立する。テナントライフサイクル管理（作成・更新・一時停止・削除）を行い、テナントイベントを Kafka で配信する。
Rust での実装を定義する。

## 概要

system tier の Tenant Server は以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| テナント管理 | テナントの作成・更新・一時停止・削除（ライフサイクル管理） |
| テナントプロビジョニング | 新規テナント作成時に Keycloak realm・DB スキーマ・初期設定を自動構成 |
| テナントメンバー管理 | テナントへのユーザー追加・削除・ロール割り当て |
| テナント設定管理 | テナント固有の設定値（config サーバー連携） |
| テナントイベント配信 | テナント作成・停止・削除イベントを Kafka `k1s0.system.tenant.events.v1` で配信 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/system/server/rust/tenant/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) および [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| テナント識別子 | UUID v4 + name（例: `acme-corp`）の 2 つの識別子を持つ。name は一意制約 |
| Keycloak 連携 | テナント作成時に Keycloak Admin API で realm を自動作成。realm 名は `k1s0-{tenant_name}` |
| DB 分離戦略 | PostgreSQL スキーマ分離（テナントごとに `tenant_{tenant_id}` スキーマ）。共有 DB クラスター |
| ステータス遷移 | provisioning -> active -> suspended -> deleted |
| プロビジョニング | Saga パターンで実装（Keycloak realm 作成 -> DB スキーマ作成 -> 初期設定投入 -> アクティベーション） |
| Kafka オプショナル | Kafka 未設定時もテナント管理は動作する。イベント配信のみスキップ |
| RBAC | `tenants/admin`（停止・削除・メンバー削除）/ `tenants/write`（作成・更新・メンバー追加）/ `tenants/read`（読み取り専用） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_TENANT_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/tenants` | テナント作成 | `tenants/write` |
| GET | `/api/v1/tenants` | テナント一覧 | `tenants/read` |
| GET | `/api/v1/tenants/:id` | テナント詳細 | `tenants/read` |
| PUT | `/api/v1/tenants/:id` | テナント更新 | `tenants/write` |
| POST | `/api/v1/tenants/:id/suspend` | テナント停止 | `tenants/admin` |
| POST | `/api/v1/tenants/:id/activate` | テナント再開 | `tenants/admin` |
| DELETE | `/api/v1/tenants/:id` | テナント削除 | `tenants/admin` |
| GET | `/api/v1/tenants/:id/members` | テナントメンバー一覧 | `tenants/read` |
| POST | `/api/v1/tenants/:id/members` | テナントメンバー追加 | `tenants/write` |
| DELETE | `/api/v1/tenants/:id/members/:user_id` | テナントメンバー削除 | `tenants/admin` |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/tenants

テナントを作成する。Saga パターンで Keycloak realm 作成・DB スキーマ作成・初期設定投入を実行し、成功時にステータスを `active` に遷移する。

**リクエスト**

```json
{
  "name": "acme-corp",
  "display_name": "Acme Corporation",
  "plan": "enterprise",
  "owner_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `name` | string | Yes | テナント名（URL フレンドリー、一意制約） |
| `display_name` | string | Yes | テナント表示名 |
| `plan` | string | Yes | 契約プラン（`free` / `starter` / `professional` / `enterprise`） |
| `owner_id` | string (UUID) | No | オーナーユーザー ID |

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "acme-corp",
  "display_name": "Acme Corporation",
  "status": "provisioning",
  "plan": "enterprise",
  "settings": {},
  "keycloak_realm": "k1s0-acme-corp",
  "db_schema": "tenant_550e8400e29b41d4a716446655440000",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_TENANT_VALIDATION_ERROR",
    "message": "invalid tenant id: ...",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_TENANT_NAME_CONFLICT",
    "message": "tenant name already exists: acme-corp",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/tenants

テナント一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "tenants": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "acme-corp",
      "display_name": "Acme Corporation",
      "status": "active",
      "plan": "enterprise",
      "settings": {},
      "keycloak_realm": "k1s0-acme-corp",
      "db_schema": "tenant_550e8400e29b41d4a716446655440000",
      "created_at": "2026-02-23T10:00:00.000+00:00",
      "updated_at": "2026-02-23T10:00:00.000+00:00"
    }
  ],
  "total_count": 42,
  "page": 1,
  "page_size": 20
}
```

#### GET /api/v1/tenants/:id

ID 指定でテナントの詳細を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "acme-corp",
  "display_name": "Acme Corporation",
  "status": "active",
  "plan": "enterprise",
  "settings": {},
  "keycloak_realm": "k1s0-acme-corp",
  "db_schema": "tenant_550e8400e29b41d4a716446655440000",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T10:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_TENANT_NOT_FOUND",
    "message": "tenant not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### PUT /api/v1/tenants/:id

テナント情報を更新する。更新可能フィールドは `display_name` と `plan`。

**リクエスト**

```json
{
  "display_name": "Acme Corp International",
  "plan": "enterprise"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `display_name` | string | Yes | テナント表示名 |
| `plan` | string | Yes | 契約プラン（`free` / `starter` / `professional` / `enterprise`） |

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "acme-corp",
  "display_name": "Acme Corp International",
  "status": "active",
  "plan": "enterprise",
  "settings": {},
  "keycloak_realm": "k1s0-acme-corp",
  "db_schema": "tenant_550e8400e29b41d4a716446655440000",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T11:00:00.000+00:00"
}
```

#### POST /api/v1/tenants/:id/suspend

テナントを一時停止する。`active` ステータスのテナントのみ停止可能。Kafka でイベントを配信する。

リクエストボディは不要。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "acme-corp",
  "display_name": "Acme Corporation",
  "status": "suspended",
  "plan": "enterprise",
  "settings": {},
  "keycloak_realm": "k1s0-acme-corp",
  "db_schema": "tenant_550e8400e29b41d4a716446655440000",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T12:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_TENANT_INVALID_STATUS",
    "message": "invalid status transition",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/tenants/:id/activate

一時停止中のテナントを再開する。`suspended` ステータスのテナントのみ再開可能。

リクエストボディは不要。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "acme-corp",
  "display_name": "Acme Corporation",
  "status": "active",
  "plan": "enterprise",
  "settings": {},
  "keycloak_realm": "k1s0-acme-corp",
  "db_schema": "tenant_550e8400e29b41d4a716446655440000",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T13:00:00.000+00:00"
}
```

#### DELETE /api/v1/tenants/:id

テナントを論理削除する。Keycloak realm の無効化と DB スキーマのアーカイブを Saga パターンで実行する。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "acme-corp",
  "display_name": "Acme Corporation",
  "status": "deleted",
  "plan": "enterprise",
  "settings": {},
  "keycloak_realm": "k1s0-acme-corp",
  "db_schema": "tenant_550e8400e29b41d4a716446655440000",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T14:00:00.000+00:00"
}
```

#### GET /api/v1/tenants/:id/members

テナントに所属するメンバー一覧を取得する。

**レスポンス（200 OK）**

```json
{
  "members": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440010",
      "user_id": "660e8400-e29b-41d4-a716-446655440001",
      "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
      "role": "admin",
      "joined_at": "2026-02-23T10:05:00.000+00:00"
    },
    {
      "id": "aa0e8400-e29b-41d4-a716-446655440011",
      "user_id": "770e8400-e29b-41d4-a716-446655440002",
      "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
      "role": "member",
      "joined_at": "2026-02-23T10:10:00.000+00:00"
    }
  ]
}
```

#### POST /api/v1/tenants/:id/members

テナントにメンバーを追加する。Keycloak realm にもユーザーを登録する。

**リクエスト**

```json
{
  "user_id": "880e8400-e29b-41d4-a716-446655440003",
  "role": "member"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `user_id` | string (UUID) | Yes | 追加するユーザーの ID |
| `role` | string | Yes | テナント内でのロール（`owner` / `admin` / `member` / `viewer`） |

**レスポンス（201 Created）**

```json
{
  "id": "bb0e8400-e29b-41d4-a716-446655440012",
  "user_id": "880e8400-e29b-41d4-a716-446655440003",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "role": "member",
  "joined_at": "2026-02-23T14:00:00.000+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_TENANT_MEMBER_CONFLICT",
    "message": "member already exists",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### DELETE /api/v1/tenants/:id/members/:user_id

テナントからメンバーを削除する。Keycloak realm からもユーザーを削除する。

**レスポンス（204 No Content）**

レスポンスボディなし。

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_TENANT_NOT_FOUND` | 404 | 指定されたテナントが見つからない |
| `SYS_TENANT_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー（不正な ID フォーマット等） |
| `SYS_TENANT_NAME_CONFLICT` | 409 | テナント名が既に使用されている |
| `SYS_TENANT_INVALID_STATUS` | 400 | 許可されていないステータス遷移 |
| `SYS_TENANT_MEMBER_CONFLICT` | 409 | ユーザーが既にテナントメンバーである |
| `SYS_TENANT_MEMBER_NOT_FOUND` | 404 | 指定されたテナントメンバーが見つからない |
| `SYS_TENANT_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

カノニカル定義ファイル: `api/proto/k1s0/system/tenant/v1/tenant.proto`

> **注意**: gRPC の RPC セットは REST エンドポイントと対称ではない。`GetProvisioningStatus` は gRPC 専用（REST エンドポイントなし）。`UpdateTenant`・`SuspendTenant`・`ActivateTenant`・`DeleteTenant` は REST のみ（gRPC では未定義）。

```protobuf
syntax = "proto3";
package k1s0.system.tenant.v1;

import "google/protobuf/timestamp.proto";
import "k1s0/system/common/v1/types.proto";

service TenantService {
  rpc CreateTenant(CreateTenantRequest) returns (CreateTenantResponse);
  rpc GetTenant(GetTenantRequest) returns (GetTenantResponse);
  rpc ListTenants(ListTenantsRequest) returns (ListTenantsResponse);
  rpc AddMember(AddMemberRequest) returns (AddMemberResponse);
  rpc RemoveMember(RemoveMemberRequest) returns (RemoveMemberResponse);
  rpc GetProvisioningStatus(GetProvisioningStatusRequest) returns (GetProvisioningStatusResponse);
}

message CreateTenantRequest {
  string name = 1;
  string display_name = 2;
  string owner_user_id = 3;
  string plan = 4;
}

message CreateTenantResponse {
  Tenant tenant = 1;
}

message GetTenantRequest {
  string tenant_id = 1;
}

message GetTenantResponse {
  Tenant tenant = 1;
}

message ListTenantsRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
}

message ListTenantsResponse {
  repeated Tenant tenants = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message AddMemberRequest {
  string tenant_id = 1;
  string user_id = 2;
  string role = 3;
}

message AddMemberResponse {
  TenantMember member = 1;
}

message RemoveMemberRequest {
  string tenant_id = 1;
  string user_id = 2;
}

message RemoveMemberResponse {
  bool success = 1;
}

message GetProvisioningStatusRequest {
  string job_id = 1;
}

message GetProvisioningStatusResponse {
  ProvisioningJob job = 1;
}

message Tenant {
  string id = 1;
  string name = 2;
  string display_name = 3;
  string status = 4;
  string plan = 5;
  google.protobuf.Timestamp created_at = 6;
}

message TenantMember {
  string id = 1;
  string tenant_id = 2;
  string user_id = 3;
  string role = 4;
  google.protobuf.Timestamp joined_at = 5;
}

message ProvisioningJob {
  string id = 1;
  string tenant_id = 2;
  string status = 3;
  string current_step = 4;
  string error_message = 5;
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp updated_at = 7;
}
```

---

## テナント状態遷移

### ステータス一覧

| ステータス値 | 説明 |
| --- | --- |
| `provisioning` | テナント作成直後。プロビジョニング処理中 |
| `active` | プロビジョニング完了。正常稼働中 |
| `suspended` | 一時停止中。テナントリソースへのアクセスが制限される |
| `deleted` | 論理削除済み。一定期間後に物理削除 |

ステータス値は小文字の文字列として返される。

### 状態遷移図

```
  provisioning ──▶ active ──▶ suspended
       │              │            │
       │              │            │ activate
       │              │            ▼
       │              │        active (復帰)
       │              │
       │              └──▶ deleted (終端)
       │
       └──▶ deleted (プロビジョニング失敗時のロールバック後)
```

### プロビジョニング Saga ステップ

テナント作成時のプロビジョニングは Saga パターンで実行する。各ステップには補償トランザクションが定義されており、失敗時にはロールバックを行う。

```
Step 1: Keycloak realm 作成
  補償: Keycloak realm 削除
    │
Step 2: PostgreSQL スキーマ作成
  補償: PostgreSQL スキーマ削除
    │
Step 3: 初期設定投入（config サーバー連携）
  補償: 設定削除
    │
Step 4: テナントステータスを active に遷移
  補償: テナントステータスを deleted に遷移
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Tenant`, `TenantMember`, `TenantProvisioningJob`, `TenantStatus` | エンティティ定義・状態遷移 |
| domain/repository | `TenantRepository`, `TenantMemberRepository` | リポジトリトレイト |
| domain/service | `TenantDomainService` | テナント名重複チェック、ステータス遷移バリデーション |
| usecase | `CreateTenantUseCase`, `GetTenantUseCase`, `ListTenantsUseCase`, `UpdateTenantUseCase`, `SuspendTenantUseCase`, `ActivateTenantUseCase`, `DeleteTenantUseCase`, `ListMembersUseCase`, `AddMemberUseCase`, `RemoveMemberUseCase`, `GetProvisioningStatusUseCase` | ユースケース |
| adapter/handler | REST ハンドラー, gRPC ハンドラー | プロトコル変換（axum / tonic） |
| adapter/gateway | `KeycloakAdminClient` | Keycloak Admin API クライアント（realm 管理） |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `TenantPostgresRepository`, `TenantMemberPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/messaging | `TenantEventPublisher`, `TenantKafkaProducer` | Kafka プロデューサー（テナントイベント配信） |

### ドメインモデル

#### Tenant

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | テナントの一意識別子 |
| `name` | String | テナント名（URL フレンドリー、一意制約） |
| `display_name` | String | テナント表示名 |
| `status` | TenantStatus | テナントステータス（`provisioning` / `active` / `suspended` / `deleted`） |
| `plan` | String | 契約プラン（`free` / `starter` / `professional` / `enterprise`） |
| `settings` | JSON | テナント固有の設定値（JSON オブジェクト） |
| `keycloak_realm` | Option\<String\> | Keycloak realm 名（`k1s0-{name}`） |
| `db_schema` | Option\<String\> | PostgreSQL スキーマ名（`tenant_{id}`） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### TenantMember

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | メンバーレコードの一意識別子 |
| `user_id` | UUID | ユーザーの一意識別子 |
| `tenant_id` | UUID | テナントの一意識別子 |
| `role` | String | テナント内でのロール（`owner` / `admin` / `member` / `viewer`） |
| `joined_at` | DateTime\<Utc\> | テナント参加日時 |

#### ProvisioningJob

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ジョブの一意識別子 |
| `tenant_id` | UUID | 対象テナントの一意識別子 |
| `status` | ProvisioningStatus | ジョブステータス（`pending` / `running` / `completed` / `failed`） |
| `current_step` | Option\<String\> | 現在実行中のステップ名 |
| `error_message` | Option\<String\> | エラーメッセージ（失敗時） |
| `created_at` | DateTime\<Utc\> | ジョブ開始日時 |
| `updated_at` | DateTime\<Utc\> | 最終更新日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (tenant_handler.rs)         │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  create / list / get / update            │   │
                    │  │  suspend / activate / delete             │   │
                    │  │  list_members / add_member / remove      │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (tenant_grpc.rs)            │   │
                    │  │  TenantService impl                      │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ Gateway: KeycloakAdminClient             │   │
                    │  │  create_realm / delete_realm             │   │
                    │  │  add_user / remove_user                  │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateTenantUseCase / GetTenantUseCase /       │
                    │  ListTenantsUseCase / UpdateTenantUseCase /     │
                    │  SuspendTenantUseCase / ActivateTenantUseCase / │
                    │  DeleteTenantUseCase / ListMembersUseCase /     │
                    │  AddMemberUseCase / RemoveMemberUseCase /       │
                    │  GetProvisioningStatusUseCase                   │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────────┐         ┌──────────▼──────────────────┐   │
    │  domain/entity      │         │ domain/repository           │   │
    │  Tenant,            │         │ TenantRepository            │   │
    │  TenantMember,      │         │ TenantMemberRepository      │   │
    │  TenantStatus,      │         │ (trait)                     │   │
    │  ProvisioningJob    │         └──────────┬─────────────────┘   │
    ├─────────────────────┤                    │                     │
    │  domain/service     │                    │                     │
    │  TenantDomainService│                    │                     │
    └─────────────────────┘                    │                     │
                                               │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ TenantPostgres-        │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  │ (events)     │  │ TenantMemberPostgres-  │  │
                    │  │              │  │ Repository             │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ Config       │  │ Database               │  │
                    │  │ Loader       │  │ Config                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "tenant-server"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051

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
  topic: "k1s0.system.tenant.events.v1"

keycloak:
  base_url: "http://keycloak.k1s0-system.svc.cluster.local:8080"
  admin_realm: "master"
  client_id: "admin-cli"
  client_secret: ""
```

---

## デプロイ

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。tenant-server 固有の values は以下の通り。

```yaml
# values-tenant.yaml（infra/helm/services/system/tenant/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/tenant-server
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/tenant/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/system/keycloak/admin"
      key: "client_secret"
      mountPath: "/vault/secrets/keycloak-secret"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/tenant/database` |
| Keycloak Admin Secret | `secret/data/k1s0/system/keycloak/admin` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-tenant-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-tenant-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-saga-server.md](../saga/server.md) -- Saga パターンによるプロビジョニング
- [REST-API設計.md](../../architecture/api/REST-API設計.md) -- D-007 統一エラーレスポンス
