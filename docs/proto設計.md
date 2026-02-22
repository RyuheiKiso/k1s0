# Proto 設計

k1s0 における Protobuf / gRPC のサービス定義・共通型・Kafka イベントスキーマ・コード生成パイプラインを定義する。
API 設計の全体像は [API設計.md](API設計.md) D-009 / D-010 を参照。

## 概要

### 採用目的

| 目的 | 説明 |
| --- | --- |
| サービス間高速通信 | HTTP/2 ベースのバイナリプロトコルにより、REST API 比で低レイテンシ・高スループットを実現する |
| 型安全なインターフェース | Protobuf スキーマから Go / Rust / TypeScript のコードを自動生成し、型不一致を防止する |
| スキーマ進化の管理 | buf による lint・破壊的変更検出で、安全なスキーマ進化を保証する |
| Kafka イベントスキーマの統一 | メッセージング基盤のイベント型も Protobuf で定義し、Schema Registry で互換性を管理する |

### バージョニング戦略

proto パッケージは [API設計.md](API設計.md) D-009 の命名規則に従い、メジャーバージョンをパッケージ名に含める。

```
k1s0.{tier}.{domain}.v{major}
```

初期バージョンは `v1` とし、後方互換性を破壊する変更が必要な場合のみ `v2` パッケージを新設する。

### 言語サポート

| 言語 | コード生成ツール | 用途 |
| --- | --- | --- |
| Go | `protoc-gen-go` + `protoc-gen-go-grpc` | Go サーバー・クライアント |
| Rust | `tonic-build` | Rust サーバー・クライアント |
| TypeScript | `ts-proto` | TypeScript ライブラリ・BFF |

---

## ディレクトリ構造

### プロジェクトルート共有定義

プロジェクトルート `api/proto/` にはサービス間で共有される型定義・イベント定義を配置する。

```
api/proto/
├── buf.yaml                              # buf 設定（lint・breaking change 検出）
├── buf.gen.yaml                          # コード生成設定
├── buf.lock                              # 依存ロック
└── k1s0/
    ├── system/
    │   └── common/
    │       └── v1/
    │           ├── types.proto           # Pagination, Timestamp 等の共通型
    │           └── event_metadata.proto  # イベントメタデータ
    └── event/
        ├── system/
        │   └── auth/
        │       └── v1/
        │           └── auth_events.proto # 認証系イベント
        ├── business/
        │   └── accounting/
        │       └── v1/
        │           └── entry_event.proto
        └── service/
            ├── order/
            │   └── v1/
            │       └── order_event.proto
            └── inventory/
                └── v1/
                    └── inventory_event.proto
```

### サービス固有定義

各サービス内の `api/proto/` にはそのサービス固有の gRPC サービス定義を配置する。

```
# auth-server
{auth-server}/api/proto/
└── k1s0/
    └── system/
        └── auth/
            └── v1/
                └── auth.proto            # AuthService gRPC 定義

# config-server
{config-server}/api/proto/
└── k1s0/
    └── system/
        └── config/
            └── v1/
                └── config.proto          # ConfigService gRPC 定義

# saga-server
api/proto/k1s0/system/saga/v1/
└── saga.proto                            # SagaService gRPC 定義（共有 proto）
```

---

## 共通メッセージ型（common.proto）

全 Tier で共有する型を `k1s0.system.common.v1` パッケージに定義する。

### types.proto

```protobuf
// api/proto/k1s0/system/common/v1/types.proto
syntax = "proto3";
package k1s0.system.common.v1;

option go_package = "github.com/k1s0-platform/api/gen/go/k1s0/system/common/v1;commonv1";

// Pagination はページネーションリクエストパラメータ。
message Pagination {
  int32 page = 1;       // ページ番号（1始まり）
  int32 page_size = 2;  // 1ページあたりの件数
}

// PaginationResult はページネーション結果。
message PaginationResult {
  int32 total_count = 1;  // 全件数
  int32 page = 2;         // 現在のページ番号
  int32 page_size = 3;    // 1ページあたりの件数
  bool has_next = 4;      // 次ページの有無
}

// Timestamp は時刻情報。google.protobuf.Timestamp と互換。
message Timestamp {
  int64 seconds = 1;  // Unix epoch からの秒数
  int32 nanos = 2;    // ナノ秒（0-999999999）
}
```

### event_metadata.proto

```protobuf
// api/proto/k1s0/system/common/v1/event_metadata.proto
syntax = "proto3";
package k1s0.system.common.v1;

option go_package = "github.com/k1s0-platform/api/gen/go/k1s0/system/common/v1;commonv1";

// EventMetadata は全イベントに付与する共通メタデータ。
message EventMetadata {
  string event_id = 1;        // UUID
  string event_type = 2;      // e.g., "auth.audit.recorded"
  string source = 3;          // e.g., "auth-server"
  int64 timestamp = 4;        // Unix timestamp (ms)
  string trace_id = 5;        // 分散トレース ID
  string correlation_id = 6;  // 業務相関 ID
  int32 schema_version = 7;   // スキーマバージョン
}
```

---

## 認証サービス定義（auth.proto）

パッケージ: `k1s0.system.auth.v1`

Go/Rust の既存 gRPC ハンドラー実装に完全対応するサービス定義。

```protobuf
// {auth-server}/api/proto/k1s0/system/auth/v1/auth.proto
syntax = "proto3";
package k1s0.system.auth.v1;

option go_package = "github.com/k1s0-platform/system-server-go-auth/gen/go/k1s0/system/auth/v1;authv1";

import "k1s0/system/common/v1/types.proto";

// AuthService は JWT トークン検証・ユーザー情報管理・パーミッション確認を提供する。
service AuthService {
  // JWT トークン検証
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);

  // ユーザー情報取得
  rpc GetUser(GetUserRequest) returns (GetUserResponse);

  // ユーザー一覧取得
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);

  // ユーザーロール取得
  rpc GetUserRoles(GetUserRolesRequest) returns (GetUserRolesResponse);

  // パーミッション確認
  rpc CheckPermission(CheckPermissionRequest) returns (CheckPermissionResponse);
}

// AuditService は監査ログの記録・検索を提供する。
service AuditService {
  // 監査ログ記録
  rpc RecordAuditLog(RecordAuditLogRequest) returns (RecordAuditLogResponse);

  // 監査ログ検索
  rpc SearchAuditLogs(SearchAuditLogsRequest) returns (SearchAuditLogsResponse);
}

// ============================================================
// Token Validation
// ============================================================

message ValidateTokenRequest {
  string token = 1;
}

message ValidateTokenResponse {
  bool valid = 1;
  TokenClaims claims = 2;
  string error_message = 3;  // valid == false の場合のエラー理由
}

message TokenClaims {
  string sub = 1;                                    // ユーザー UUID
  string iss = 2;                                    // Issuer
  string aud = 3;                                    // Audience
  int64 exp = 4;                                     // 有効期限（Unix epoch）
  int64 iat = 5;                                     // 発行日時（Unix epoch）
  string jti = 6;                                    // Token ID
  string preferred_username = 7;                     // ユーザー名
  string email = 8;                                  // メールアドレス
  RealmAccess realm_access = 9;                      // グローバルロール
  map<string, ClientRoles> resource_access = 10;     // サービス固有ロール
  repeated string tier_access = 11;                  // アクセス可能 Tier
}

message RealmAccess {
  repeated string roles = 1;
}

message ClientRoles {
  repeated string roles = 1;
}

// ============================================================
// User
// ============================================================

message GetUserRequest {
  string user_id = 1;
}

message GetUserResponse {
  User user = 1;
}

message ListUsersRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
  string search = 2;                                 // ユーザー名・メールで部分一致検索
  optional bool enabled = 3;                         // 有効/無効フィルタ
}

message ListUsersResponse {
  repeated User users = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message User {
  string id = 1;
  string username = 2;
  string email = 3;
  string first_name = 4;
  string last_name = 5;
  bool enabled = 6;
  bool email_verified = 7;
  k1s0.system.common.v1.Timestamp created_at = 8;
  map<string, StringList> attributes = 9;            // カスタム属性（部署, 社員番号等）
}

message StringList {
  repeated string values = 1;
}

// ============================================================
// Roles
// ============================================================

message GetUserRolesRequest {
  string user_id = 1;
}

message GetUserRolesResponse {
  string user_id = 1;
  repeated Role realm_roles = 2;                     // グローバルロール一覧
  map<string, RoleList> client_roles = 3;            // クライアント別ロール
}

message Role {
  string id = 1;
  string name = 2;
  string description = 3;
}

message RoleList {
  repeated Role roles = 1;
}

// ============================================================
// Permission Check
// ============================================================

message CheckPermissionRequest {
  string user_id = 1;
  string permission = 2;     // read, write, delete, admin
  string resource = 3;       // users, auth_config, audit_logs, etc.
  repeated string roles = 4; // JWT Claims から取得したロール一覧
}

message CheckPermissionResponse {
  bool allowed = 1;
  string reason = 2;         // 拒否理由（allowed == false の場合）
}

// ============================================================
// Audit Log
// ============================================================

message RecordAuditLogRequest {
  string event_type = 1;           // LOGIN_SUCCESS, LOGIN_FAILURE, TOKEN_VALIDATE, PERMISSION_DENIED 等
  string user_id = 2;
  string ip_address = 3;
  string user_agent = 4;
  string resource = 5;             // アクセス対象リソース
  string action = 6;               // HTTP メソッドまたは gRPC メソッド名
  string result = 7;               // SUCCESS / FAILURE
  google.protobuf.Struct detail = 8; // 操作の詳細情報（client_id, grant_type 等）
  string resource_id = 9;          // 操作対象リソースの ID
  string trace_id = 10;            // OpenTelemetry トレース ID
}

message RecordAuditLogResponse {
  string id = 1;                                     // 監査ログ UUID
  k1s0.system.common.v1.Timestamp created_at = 2;
}

message SearchAuditLogsRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
  string user_id = 2;
  string event_type = 3;
  k1s0.system.common.v1.Timestamp from = 4;
  k1s0.system.common.v1.Timestamp to = 5;
  string result = 6;               // SUCCESS / FAILURE
}

message SearchAuditLogsResponse {
  repeated AuditLog logs = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message AuditLog {
  string id = 1;
  string event_type = 2;
  string user_id = 3;
  string ip_address = 4;
  string user_agent = 5;
  string resource = 6;
  string action = 7;
  string result = 8;
  google.protobuf.Struct detail = 9;               // 操作の詳細情報（変更前後の値等）
  k1s0.system.common.v1.Timestamp created_at = 10;
  string resource_id = 11;                         // 操作対象リソースの ID
  string trace_id = 12;                            // OpenTelemetry トレース ID
}
```

### RPC と既存ハンドラーの対応

| RPC | Go ハンドラー | Rust ハンドラー | 説明 |
| --- | --- | --- | --- |
| `AuthService.ValidateToken` | `AuthGRPCService.ValidateToken` | `auth_handler::validate_token` (REST) | JWT 署名・有効期限・issuer・audience 検証 |
| `AuthService.GetUser` | `AuthGRPCService.GetUser` | `auth_handler::get_user` (REST) | Keycloak Admin API 経由でユーザー情報取得 |
| `AuthService.ListUsers` | `AuthGRPCService.ListUsers` | `auth_handler::list_users` (REST) | ページネーション付きユーザー一覧 |
| `AuthService.GetUserRoles` | `AuthGRPCService.GetUserRoles` | `auth_handler::get_user_roles` (REST) | ユーザーのロール一覧（realm + client） |
| `AuthService.CheckPermission` | `AuthGRPCService.CheckPermission` | `auth_handler::check_permission` (REST) | RBAC パーミッション判定 |
| `AuditService.RecordAuditLog` | `AuditGRPCService.RecordAuditLog` | `auth_handler::record_audit_log` (REST) | 監査ログエントリ記録 |
| `AuditService.SearchAuditLogs` | `AuditGRPCService.SearchAuditLogs` | `auth_handler::search_audit_logs` (REST) | 監査ログ検索 |

---

## 設定管理サービス定義（config.proto）

パッケージ: `k1s0.system.config.v1`

Go/Rust の既存 gRPC ハンドラー実装に完全対応するサービス定義。

```protobuf
// {config-server}/api/proto/k1s0/system/config/v1/config.proto
syntax = "proto3";
package k1s0.system.config.v1;

option go_package = "github.com/k1s0-platform/system-server-go-config/gen/go/k1s0/system/config/v1;configv1";

import "k1s0/system/common/v1/types.proto";

// ConfigService は設定値の取得・更新・削除・監視を提供する。
service ConfigService {
  // 設定値取得
  rpc GetConfig(GetConfigRequest) returns (GetConfigResponse);

  // namespace 内の設定値一覧取得
  rpc ListConfigs(ListConfigsRequest) returns (ListConfigsResponse);

  // 設定値更新
  rpc UpdateConfig(UpdateConfigRequest) returns (UpdateConfigResponse);

  // 設定値削除
  rpc DeleteConfig(DeleteConfigRequest) returns (DeleteConfigResponse);

  // サービス向け設定一括取得
  rpc GetServiceConfig(GetServiceConfigRequest) returns (GetServiceConfigResponse);

  // 設定変更の監視（Server-Side Streaming）
  rpc WatchConfig(WatchConfigRequest) returns (stream ConfigChangeEvent);
}

// ============================================================
// ConfigEntry
// ============================================================

message ConfigEntry {
  string id = 1;                                     // UUID
  string namespace = 2;                              // e.g., "system.auth.database"
  string key = 3;                                    // e.g., "max_connections"
  bytes value = 4;                                   // JSON エンコード済みの値
  int32 version = 5;                                 // 楽観的排他制御用バージョン
  string description = 6;
  string created_by = 7;
  string updated_by = 8;
  k1s0.system.common.v1.Timestamp created_at = 9;
  k1s0.system.common.v1.Timestamp updated_at = 10;
}

// ============================================================
// GetConfig
// ============================================================

message GetConfigRequest {
  string namespace = 1;
  string key = 2;
}

message GetConfigResponse {
  ConfigEntry entry = 1;
}

// ============================================================
// ListConfigs
// ============================================================

message ListConfigsRequest {
  string namespace = 1;
  k1s0.system.common.v1.Pagination pagination = 2;
  string search = 3;           // キー名の部分一致検索
}

message ListConfigsResponse {
  repeated ConfigEntry entries = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

// ============================================================
// UpdateConfig
// ============================================================

message UpdateConfigRequest {
  string namespace = 1;
  string key = 2;
  bytes value = 3;              // JSON エンコード済みの値
  int32 version = 4;            // 楽観的排他制御用（現在のバージョン番号）
  string description = 5;
  string updated_by = 6;
}

message UpdateConfigResponse {
  ConfigEntry entry = 1;
}

// ============================================================
// DeleteConfig
// ============================================================

message DeleteConfigRequest {
  string namespace = 1;
  string key = 2;
  string deleted_by = 3;
}

message DeleteConfigResponse {
  bool success = 1;
}

// ============================================================
// GetServiceConfig
// ============================================================

message GetServiceConfigRequest {
  string service_name = 1;
  string environment = 2;      // dev | staging | prod
}

message GetServiceConfigResponse {
  map<string, string> configs = 1;  // flattened key-value pairs
}

// ============================================================
// WatchConfig（Server-Side Streaming）
// ============================================================

message WatchConfigRequest {
  repeated string namespaces = 1;  // 監視対象 namespace（空の場合は全件）
}

message ConfigChangeEvent {
  string namespace = 1;
  string key = 2;
  bytes old_value = 3;                               // 変更前の値（JSON エンコード済み）
  bytes new_value = 4;                               // 変更後の値（JSON エンコード済み）
  int32 old_version = 5;
  int32 new_version = 6;
  string changed_by = 7;
  string change_type = 8;                            // CREATED, UPDATED, DELETED
  k1s0.system.common.v1.Timestamp changed_at = 9;
}
```

### RPC と既存ハンドラーの対応

| RPC | Go ハンドラー | Rust ハンドラー | 説明 |
| --- | --- | --- | --- |
| `ConfigService.GetConfig` | `ConfigGRPCService.GetConfig` | `ConfigGrpcService.get_config` | namespace + key で設定値取得 |
| `ConfigService.ListConfigs` | `ConfigGRPCService.ListConfigs` | `ConfigGrpcService.list_configs` | namespace 内の設定値一覧（ページネーション付き） |
| `ConfigService.UpdateConfig` | `ConfigGRPCService.UpdateConfig` | `ConfigGrpcService.update_config` | 楽観的排他制御付き設定値更新 |
| `ConfigService.DeleteConfig` | `ConfigGRPCService.DeleteConfig` | `ConfigGrpcService.delete_config` | 設定値削除（sys_admin 権限） |
| `ConfigService.GetServiceConfig` | `ConfigGRPCService.GetServiceConfig` | `ConfigGrpcService.get_service_config` | サービス名で設定一括取得 |
| `ConfigService.WatchConfig` | `ConfigGRPCService.WatchConfig` (未実装) | `ConfigGrpcService.watch_config` (実装済み) | 設定変更のリアルタイム監視 |

---

## Saga サービス定義（saga.proto）

パッケージ: `k1s0.system.saga.v1`

分散トランザクション（Saga パターン）のオーケストレーション機能を提供するサービス定義。
定義ファイルは `api/proto/k1s0/system/saga/v1/saga.proto` に配置する（共有 proto）。

```protobuf
// api/proto/k1s0/system/saga/v1/saga.proto
syntax = "proto3";
package k1s0.system.saga.v1;

option go_package = "github.com/k1s0-platform/system-server-go-saga/gen/go/k1s0/system/saga/v1;sagav1";

import "k1s0/system/common/v1/types.proto";

// SagaService は Saga オーケストレーション機能を提供する。
service SagaService {
  // Saga 開始（非同期実行）
  rpc StartSaga(StartSagaRequest) returns (StartSagaResponse);

  // Saga 詳細取得（ステップログ含む）
  rpc GetSaga(GetSagaRequest) returns (GetSagaResponse);

  // Saga 一覧取得
  rpc ListSagas(ListSagasRequest) returns (ListSagasResponse);

  // Saga キャンセル
  rpc CancelSaga(CancelSagaRequest) returns (CancelSagaResponse);

  // ワークフロー登録（YAML 文字列）
  rpc RegisterWorkflow(RegisterWorkflowRequest) returns (RegisterWorkflowResponse);

  // ワークフロー一覧取得
  rpc ListWorkflows(ListWorkflowsRequest) returns (ListWorkflowsResponse);
}
```

### RPC と既存ハンドラーの対応

| RPC | Rust ハンドラー | 説明 |
| --- | --- | --- |
| `SagaService.StartSaga` | `SagaGrpcService.start_saga` | ワークフロー名・ペイロードで Saga を開始 |
| `SagaService.GetSaga` | `SagaGrpcService.get_saga` | Saga ID でステップログを含む詳細取得 |
| `SagaService.ListSagas` | `SagaGrpcService.list_sagas` | ページネーション・フィルタ付き一覧取得 |
| `SagaService.CancelSaga` | `SagaGrpcService.cancel_saga` | 実行中 Saga のキャンセル |
| `SagaService.RegisterWorkflow` | `SagaGrpcService.register_workflow` | YAML 形式のワークフロー定義を登録 |
| `SagaService.ListWorkflows` | `SagaGrpcService.list_workflows` | 登録済みワークフロー一覧取得 |

---

## buf 設定

### buf.yaml

```yaml
# api/proto/buf.yaml
version: v2
modules:
  - path: .
lint:
  use:
    - STANDARD
  except:
    - PACKAGE_VERSION_SUFFIX   # v1 パッケージを許容
breaking:
  use:
    - FILE
```

### buf.gen.yaml

```yaml
# api/proto/buf.gen.yaml
version: v2
plugins:
  # --- Go ---
  - remote: buf.build/protocolbuffers/go
    out: gen/go
    opt:
      - paths=source_relative

  - remote: buf.build/grpc/go
    out: gen/go
    opt:
      - paths=source_relative

  # --- TypeScript (ts-proto) ---
  - remote: buf.build/community/timostamm-protobuf-ts
    out: gen/ts
    opt:
      - long_type_string
```

#### Rust (tonic-build)

Rust は `buf.gen.yaml` ではなく、各サービスの `build.rs` で `tonic-build` を使用してコード生成する。

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/proto")
        .compile_protos(
            &["api/proto/k1s0/system/auth/v1/auth.proto"],
            &["api/proto", "../../api/proto"],  // 共有定義のパスを含める
        )?;
    Ok(())
}
```

---

## コード生成パイプライン

### 各言語向けの生成コマンド

#### Go

```bash
# プロジェクトルートで実行
cd api/proto
buf generate
```

生成先:

```
api/proto/gen/go/
└── k1s0/
    └── system/
        ├── common/
        │   └── v1/
        │       ├── types.pb.go
        │       └── event_metadata.pb.go
        ├── auth/
        │   └── v1/
        │       ├── auth.pb.go
        │       └── auth_grpc.pb.go
        └── config/
            └── v1/
                ├── config.pb.go
                └── config_grpc.pb.go
```

#### Rust

```bash
# 各サービスディレクトリで実行
cd regions/system/server/rust/auth
cargo build  # build.rs が tonic-build を実行
```

生成先:

```
src/proto/
├── k1s0.system.auth.v1.rs
└── k1s0.system.common.v1.rs
```

#### TypeScript

```bash
cd api/proto
buf generate
```

生成先:

```
api/proto/gen/ts/
└── k1s0/
    └── system/
        ├── common/
        │   └── v1/
        │       ├── types.ts
        │       └── event_metadata.ts
        ├── auth/
        │   └── v1/
        │       └── auth.ts
        └── config/
            └── v1/
                └── config.ts
```

### CI/CD への統合

```yaml
# .github/workflows/ci.yaml（proto 関連の抜粋）
jobs:
  proto-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bufbuild/buf-setup-action@v1

      - name: Lint
        run: buf lint api/proto

      - name: Breaking change detection
        run: buf breaking api/proto --against '.git#branch=main'

      - name: Generate
        run: |
          cd api/proto
          buf generate

      - name: Verify no diff
        run: git diff --exit-code api/proto/gen/
```

---

## Kafka イベントスキーマ（Protobuf）

[メッセージング設計.md](メッセージング設計.md) で定義されたイベントスキーマの Protobuf 定義。

### AuditLogRecordedEvent

auth-server が監査ログ記録時に Kafka トピック `k1s0.system.auth.audit.v1` に配信するイベント。

```protobuf
// api/proto/k1s0/event/system/auth/v1/auth_events.proto
syntax = "proto3";
package k1s0.event.system.auth.v1;

option go_package = "github.com/k1s0-platform/api/gen/go/k1s0/event/system/auth/v1;autheventv1";

import "k1s0/system/common/v1/event_metadata.proto";

// LoginEvent はログイン成功/失敗イベント。
message LoginEvent {
  k1s0.system.common.v1.EventMetadata metadata = 1;
  string user_id = 2;
  string username = 3;
  string client_id = 4;
  string ip_address = 5;
  string user_agent = 6;
  string result = 7;        // SUCCESS / FAILURE
  string failure_reason = 8; // 失敗時のみ
}

// TokenValidationEvent はトークン検証結果イベント。
message TokenValidationEvent {
  k1s0.system.common.v1.EventMetadata metadata = 1;
  string user_id = 2;
  string token_jti = 3;
  bool valid = 4;
  string error_message = 5;  // 検証失敗時のみ
}

// PermissionCheckEvent はパーミッション確認結果イベント。
message PermissionCheckEvent {
  k1s0.system.common.v1.EventMetadata metadata = 1;
  string user_id = 2;
  string permission = 3;
  string resource = 4;
  repeated string roles = 5;
  bool allowed = 6;
  string reason = 7;
}

// AuditLogRecordedEvent は監査ログ記録イベント。
message AuditLogRecordedEvent {
  k1s0.system.common.v1.EventMetadata metadata = 1;
  string audit_log_id = 2;
  string event_type = 3;
  string user_id = 4;
  string ip_address = 5;
  string resource = 6;
  string action = 7;
  string result = 8;
}
```

### ConfigChangedEvent

config-server が設定変更時に Kafka トピック `k1s0.system.config.changed.v1` に配信するイベント。

```protobuf
// api/proto/k1s0/event/system/config/v1/config_events.proto
syntax = "proto3";
package k1s0.event.system.config.v1;

option go_package = "github.com/k1s0-platform/api/gen/go/k1s0/event/system/config/v1;configeventv1";

import "k1s0/system/common/v1/event_metadata.proto";

// ConfigChangedEvent は設定値変更イベント。
message ConfigChangedEvent {
  k1s0.system.common.v1.EventMetadata metadata = 1;
  string namespace = 2;
  string key = 3;
  string old_value = 4;      // JSON 文字列（変更前。新規作成時は空）
  string new_value = 5;      // JSON 文字列（変更後。削除時は空）
  int32 old_version = 6;
  int32 new_version = 7;
  string change_type = 8;    // CREATED, UPDATED, DELETED
  string changed_by = 9;
}
```

### イベントと Kafka トピックの対応

| イベント型 | Kafka トピック | パーティションキー | Producer |
| --- | --- | --- | --- |
| `LoginEvent` | `k1s0.system.auth.login.v1` | `user_id` | auth-server |
| `TokenValidationEvent` | `k1s0.system.auth.audit.v1` | `user_id` | auth-server |
| `PermissionCheckEvent` | `k1s0.system.auth.audit.v1` | `user_id` | auth-server |
| `AuditLogRecordedEvent` | `k1s0.system.auth.audit.v1` | `user_id` | auth-server |
| `ConfigChangedEvent` | `k1s0.system.config.changed.v1` | `namespace` | config-server |

---

## バージョニング・後方互換性ルール

### 後方互換（バージョンアップ不要）

| 変更種別 | 説明 |
| --- | --- |
| フィールド追加 | 新しいフィールド番号で追加。既存のデシリアライズに影響なし |
| 新規 RPC メソッド追加 | サービス定義に新メソッドを追加。既存クライアントは影響なし |
| 新規 enum 値追加 | 既存の enum に新しい値を追加 |
| フィールド名変更 | ワイヤーフォーマットは番号ベースのため互換性維持 |

### 後方互換性を破壊する変更（メジャーバージョンアップ）

| 変更種別 | 説明 |
| --- | --- |
| フィールドの削除・番号変更 | 既存のデシリアライズが失敗する |
| フィールドの型変更 | ワイヤーフォーマットが変わる |
| RPC メソッドのシグネチャ変更 | リクエスト/レスポンス型の変更 |
| メッセージ名の変更 | JSON マッピング・リフレクションに影響 |

### 削除時のフィールド番号予約

フィールドを削除する場合は `reserved` で番号を予約し、再利用を防止する。

```protobuf
message Example {
  reserved 2, 5;
  reserved "old_field_name";
  string id = 1;
  string name = 3;
}
```

### buf breaking による自動検証

CI パイプラインで `buf breaking` を実行し、意図しない破壊的変更を検出する。

```bash
# main ブランチとの比較
buf breaking api/proto --against '.git#branch=main'
```

破壊的変更が検出された場合は CI が失敗する。意図的な変更であれば新しいバージョンパッケージ（`v2`）として作成する。

---

## 関連ドキュメント

- [API設計.md](API設計.md) -- gRPC サービス定義パターン (D-009)・gRPC バージョニング (D-010)
- [system-server設計.md](system-server設計.md) -- auth-server の gRPC サービス定義
- [system-config-server設計.md](system-config-server設計.md) -- config-server の gRPC サービス定義
- [認証認可設計.md](認証認可設計.md) -- JWT Claims 構造・RBAC ロール定義
- [メッセージング設計.md](メッセージング設計.md) -- Kafka トピック・イベントスキーマ
- [テンプレート仕様-APIスキーマ.md](テンプレート仕様-APIスキーマ.md) -- proto テンプレート生成仕様
