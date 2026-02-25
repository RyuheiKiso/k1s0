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

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC | tonic v0.12 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 / k1s0-telemetry |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/tenant/` |

---

## 設計方針

[認証認可設計.md](認証認可設計.md) および [メッセージング設計.md](メッセージング設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| テナント識別子 | UUID v4 + スラッグ（例: `acme-corp`）の 2 つの識別子を持つ。スラッグは一意制約 |
| Keycloak 連携 | テナント作成時に Keycloak Admin API で realm を自動作成。realm 名は `k1s0-{tenant_slug}` |
| DB 分離戦略 | PostgreSQL スキーマ分離（テナントごとに `tenant_{tenant_id}` スキーマ）。共有 DB クラスター |
| ステータス遷移 | PENDING -> ACTIVE -> SUSPENDED -> DELETED |
| プロビジョニング | Saga パターンで実装（Keycloak realm 作成 -> DB スキーマ作成 -> 初期設定投入 -> アクティベーション） |
| Kafka オプショナル | Kafka 未設定時もテナント管理は動作する。イベント配信のみスキップ |
| RBAC | `sys_admin`（全権限）/ `sys_operator`（作成・更新・削除）/ `sys_auditor`（読み取り専用） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_TENANT_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/tenants` | テナント作成 | `sys_admin` のみ |
| GET | `/api/v1/tenants` | テナント一覧 | `sys_auditor` 以上 |
| GET | `/api/v1/tenants/:id` | テナント詳細 | `sys_auditor` 以上 |
| PUT | `/api/v1/tenants/:id` | テナント更新 | `sys_operator` 以上 |
| POST | `/api/v1/tenants/:id/suspend` | テナント停止 | `sys_admin` のみ |
| POST | `/api/v1/tenants/:id/activate` | テナント再開 | `sys_admin` のみ |
| DELETE | `/api/v1/tenants/:id` | テナント削除 | `sys_admin` のみ |
| GET | `/api/v1/tenants/:id/members` | テナントメンバー一覧 | `sys_auditor` 以上 |
| POST | `/api/v1/tenants/:id/members` | テナントメンバー追加（未実装・将来対応予定） | `sys_operator` 以上 |
| DELETE | `/api/v1/tenants/:id/members/:user_id` | テナントメンバー削除（未実装・将来対応予定） | `sys_operator` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/tenants

テナントを作成する。Saga パターンで Keycloak realm 作成・DB スキーマ作成・初期設定投入を実行し、成功時にステータスを ACTIVE に遷移する。

**リクエスト**

```json
{
  "slug": "acme-corp",
  "display_name": "Acme Corporation",
  "plan": "enterprise",
  "metadata": {
    "region": "ap-northeast-1",
    "contact_email": "admin@acme.example.com"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "tenant": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "slug": "acme-corp",
    "display_name": "Acme Corporation",
    "status": "PENDING",
    "plan": "enterprise",
    "keycloak_realm": "k1s0-acme-corp",
    "db_schema": "tenant_550e8400e29b41d4a716446655440000",
    "metadata": {
      "region": "ap-northeast-1",
      "contact_email": "admin@acme.example.com"
    },
    "created_at": "2026-02-23T10:00:00.000+00:00",
    "updated_at": "2026-02-23T10:00:00.000+00:00"
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_TENANT_VALIDATION_ERROR",
    "message": "slug must be 3-63 characters, lowercase alphanumeric with hyphens",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_TENANT_SLUG_CONFLICT",
    "message": "tenant with slug 'acme-corp' already exists",
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
| `status` | string | No | - | ステータスフィルタ（PENDING / ACTIVE / SUSPENDED / DELETED） |

**レスポンス（200 OK）**

```json
{
  "tenants": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "slug": "acme-corp",
      "display_name": "Acme Corporation",
      "status": "ACTIVE",
      "plan": "enterprise",
      "keycloak_realm": "k1s0-acme-corp",
      "db_schema": "tenant_550e8400e29b41d4a716446655440000",
      "metadata": {
        "region": "ap-northeast-1",
        "contact_email": "admin@acme.example.com"
      },
      "created_at": "2026-02-23T10:00:00.000+00:00",
      "updated_at": "2026-02-23T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 42,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### GET /api/v1/tenants/:id

ID 指定でテナントの詳細を取得する。

**レスポンス（200 OK）**

```json
{
  "tenant": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "slug": "acme-corp",
    "display_name": "Acme Corporation",
    "status": "ACTIVE",
    "plan": "enterprise",
    "keycloak_realm": "k1s0-acme-corp",
    "db_schema": "tenant_550e8400e29b41d4a716446655440000",
    "metadata": {
      "region": "ap-northeast-1",
      "contact_email": "admin@acme.example.com"
    },
    "created_at": "2026-02-23T10:00:00.000+00:00",
    "updated_at": "2026-02-23T10:00:00.000+00:00"
  }
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

テナント情報を更新する。更新可能フィールドは `display_name` と `metadata` のみ。

**リクエスト**

```json
{
  "display_name": "Acme Corp International",
  "metadata": {
    "region": "ap-northeast-1",
    "contact_email": "global-admin@acme.example.com"
  }
}
```

**レスポンス（200 OK）**

```json
{
  "tenant": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "slug": "acme-corp",
    "display_name": "Acme Corp International",
    "status": "ACTIVE",
    "plan": "enterprise",
    "keycloak_realm": "k1s0-acme-corp",
    "db_schema": "tenant_550e8400e29b41d4a716446655440000",
    "metadata": {
      "region": "ap-northeast-1",
      "contact_email": "global-admin@acme.example.com"
    },
    "created_at": "2026-02-23T10:00:00.000+00:00",
    "updated_at": "2026-02-23T11:00:00.000+00:00"
  }
}
```

#### POST /api/v1/tenants/:id/suspend

テナントを一時停止する。ACTIVE ステータスのテナントのみ停止可能。停止理由を記録し、Kafka でイベントを配信する。

**リクエスト**

```json
{
  "reason": "Payment overdue for 30 days"
}
```

**レスポンス（200 OK）**

```json
{
  "tenant": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "slug": "acme-corp",
    "display_name": "Acme Corporation",
    "status": "SUSPENDED",
    "plan": "enterprise",
    "keycloak_realm": "k1s0-acme-corp",
    "db_schema": "tenant_550e8400e29b41d4a716446655440000",
    "metadata": {
      "region": "ap-northeast-1",
      "contact_email": "admin@acme.example.com"
    },
    "created_at": "2026-02-23T10:00:00.000+00:00",
    "updated_at": "2026-02-23T12:00:00.000+00:00"
  }
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_TENANT_INVALID_STATUS_TRANSITION",
    "message": "cannot suspend tenant: current status is PENDING",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/tenants/:id/activate

一時停止中のテナントを再開する。SUSPENDED ステータスのテナントのみ再開可能。

**レスポンス（200 OK）**

```json
{
  "tenant": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "slug": "acme-corp",
    "display_name": "Acme Corporation",
    "status": "ACTIVE",
    "plan": "enterprise",
    "keycloak_realm": "k1s0-acme-corp",
    "db_schema": "tenant_550e8400e29b41d4a716446655440000",
    "metadata": {
      "region": "ap-northeast-1",
      "contact_email": "admin@acme.example.com"
    },
    "created_at": "2026-02-23T10:00:00.000+00:00",
    "updated_at": "2026-02-23T13:00:00.000+00:00"
  }
}
```

#### DELETE /api/v1/tenants/:id

テナントを論理削除する。Keycloak realm の無効化と DB スキーマのアーカイブを Saga パターンで実行する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "tenant 550e8400-e29b-41d4-a716-446655440000 marked as deleted"
}
```

#### GET /api/v1/tenants/:id/members

テナントに所属するメンバー一覧を取得する。

**レスポンス（200 OK）**

```json
{
  "members": [
    {
      "user_id": "660e8400-e29b-41d4-a716-446655440001",
      "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
      "role": "admin",
      "joined_at": "2026-02-23T10:05:00.000+00:00"
    },
    {
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

**レスポンス（201 Created）**

```json
{
  "member": {
    "user_id": "880e8400-e29b-41d4-a716-446655440003",
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
    "role": "member",
    "joined_at": "2026-02-23T14:00:00.000+00:00"
  }
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_TENANT_MEMBER_ALREADY_EXISTS",
    "message": "user 880e8400-e29b-41d4-a716-446655440003 is already a member of tenant 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### DELETE /api/v1/tenants/:id/members/:user_id

テナントからメンバーを削除する。Keycloak realm からもユーザーを削除する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "member 880e8400-e29b-41d4-a716-446655440003 removed from tenant 550e8400-e29b-41d4-a716-446655440000"
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_TENANT_NOT_FOUND` | 404 | 指定されたテナントが見つからない |
| `SYS_TENANT_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー（不正なスラッグ、必須フィールド未入力等） |
| `SYS_TENANT_SLUG_CONFLICT` | 409 | スラッグが既に使用されている |
| `SYS_TENANT_INVALID_STATUS_TRANSITION` | 409 | 許可されていないステータス遷移（例: PENDING -> SUSPENDED） |
| `SYS_TENANT_MEMBER_ALREADY_EXISTS` | 409 | ユーザーが既にテナントメンバーである |
| `SYS_TENANT_MEMBER_NOT_FOUND` | 404 | 指定されたテナントメンバーが見つからない |
| `SYS_TENANT_PROVISIONING_FAILED` | 500 | プロビジョニング処理（Keycloak/DB/設定）が失敗 |
| `SYS_TENANT_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.tenant.v1;

service TenantService {
  rpc CreateTenant(CreateTenantRequest) returns (CreateTenantResponse);
  rpc GetTenant(GetTenantRequest) returns (GetTenantResponse);
  rpc ListTenants(ListTenantsRequest) returns (ListTenantsResponse);
  rpc UpdateTenant(UpdateTenantRequest) returns (UpdateTenantResponse);
  rpc SuspendTenant(SuspendTenantRequest) returns (SuspendTenantResponse);
  rpc GetTenantMembers(GetTenantMembersRequest) returns (GetTenantMembersResponse);
}

message CreateTenantRequest {
  string slug = 1;
  string display_name = 2;
  string plan = 3;
  map<string, string> metadata = 4;
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
  int32 page = 1;
  int32 page_size = 2;
  string status = 3;
}

message ListTenantsResponse {
  repeated Tenant tenants = 1;
  int32 total_count = 2;
}

message UpdateTenantRequest {
  string tenant_id = 1;
  optional string display_name = 2;
  map<string, string> metadata = 3;
}

message UpdateTenantResponse {
  Tenant tenant = 1;
}

message SuspendTenantRequest {
  string tenant_id = 1;
  string reason = 2;
}

message SuspendTenantResponse {
  Tenant tenant = 1;
}

message GetTenantMembersRequest {
  string tenant_id = 1;
}

message GetTenantMembersResponse {
  repeated TenantMember members = 1;
}

message Tenant {
  string id = 1;
  string slug = 2;
  string display_name = 3;
  string status = 4;
  string plan = 5;
  map<string, string> metadata = 6;
  string created_at = 7;
  string updated_at = 8;
}

message TenantMember {
  string user_id = 1;
  string tenant_id = 2;
  string role = 3;
  string joined_at = 4;
}
```

---

## テナント状態遷移

### ステータス一覧

| ステータス | 説明 |
| --- | --- |
| `PENDING` | テナント作成直後。プロビジョニング処理中 |
| `ACTIVE` | プロビジョニング完了。正常稼働中 |
| `SUSPENDED` | 一時停止中。テナントリソースへのアクセスが制限される |
| `DELETED` | 論理削除済み。一定期間後に物理削除 |

### 状態遷移図

```
  PENDING ──▶ ACTIVE ──▶ SUSPENDED
    │            │            │
    │            │            │ activate
    │            │            ▼
    │            │        ACTIVE (復帰)
    │            │
    │            └──▶ DELETED (終端)
    │
    └──▶ DELETED (プロビジョニング失敗時のロールバック後)
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
Step 4: テナントステータスを ACTIVE に遷移
  補償: テナントステータスを DELETED に遷移
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース・ドメインサービス）
  ^
usecase（ビジネスロジック）
  ^
adapter（ハンドラー・プレゼンター・ゲートウェイ）
  ^
infrastructure（DB接続・Kafka Producer・Keycloak Client・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Tenant`, `TenantMember`, `TenantProvisioningJob`, `TenantStatus` | エンティティ定義・状態遷移 |
| domain/repository | `TenantRepository`, `TenantMemberRepository` | リポジトリトレイト |
| domain/service | `TenantDomainService` | スラッグ重複チェック、ステータス遷移バリデーション |
| usecase | `CreateTenantUsecase`, `GetTenantUsecase`, `ListTenantsUsecase`, `UpdateTenantUsecase`, `SuspendTenantUsecase`, `ActivateTenantUsecase`, `DeleteTenantUsecase`, `AddMemberUsecase`, `RemoveMemberUsecase` | ユースケース |
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
| `slug` | String | テナントスラッグ（URL フレンドリー、一意制約） |
| `display_name` | String | テナント表示名 |
| `status` | TenantStatus | テナントステータス（PENDING / ACTIVE / SUSPENDED / DELETED） |
| `plan` | String | 契約プラン |
| `keycloak_realm` | String | Keycloak realm 名（`k1s0-{slug}`） |
| `db_schema` | String | PostgreSQL スキーマ名（`tenant_{id}`） |
| `metadata` | Map\<String, String\> | テナント固有のメタデータ |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### TenantMember

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `user_id` | UUID | ユーザーの一意識別子 |
| `tenant_id` | UUID | テナントの一意識別子 |
| `role` | String | テナント内でのロール（admin / member） |
| `joined_at` | DateTime\<Utc\> | テナント参加日時 |

#### TenantProvisioningJob

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ジョブの一意識別子 |
| `tenant_id` | UUID | 対象テナントの一意識別子 |
| `status` | String | ジョブステータス（RUNNING / COMPLETED / FAILED / ROLLED_BACK） |
| `steps` | JSON | 各ステップの実行状態 |
| `error` | Option\<String\> | エラーメッセージ（失敗時） |
| `created_at` | DateTime\<Utc\> | ジョブ開始日時 |
| `completed_at` | Option\<DateTime\<Utc\>\> | ジョブ完了日時 |

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
                    │  CreateTenantUsecase / GetTenantUsecase /       │
                    │  ListTenantsUsecase / UpdateTenantUsecase /     │
                    │  SuspendTenantUsecase / ActivateTenantUsecase / │
                    │  DeleteTenantUsecase / AddMemberUsecase /       │
                    │  RemoveMemberUsecase                            │
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

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。tenant-server 固有の values は以下の通り。

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

- [system-tenant-server-実装設計.md](system-tenant-server-実装設計.md) -- 実装設計の詳細
- [system-tenant-server-デプロイ設計.md](system-tenant-server-デプロイ設計.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

- [認証認可設計.md](認証認可設計.md) -- 認証・認可の基本方針
- [system-saga-server設計.md](system-saga-server設計.md) -- Saga パターンによるプロビジョニング
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [REST-API設計.md](REST-API設計.md) -- D-007 統一エラーレスポンス
- [メッセージング設計.md](メッセージング設計.md) -- Kafka イベント配信パターン
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [config設計.md](config設計.md) -- config.yaml スキーマ
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
