# system-vault-server 設計

> **認可モデル注記（2026-03-03更新）**: 実装では `resource/action`（例: `secrets/read`, `secrets/write`, `secrets/admin`）で判定し、ロール `sys_admin` / `sys_operator` / `sys_auditor` は middleware でそれぞれ `admin` / `write` / `read` にマッピングされます。


system tier のシークレット管理サーバー設計を定義する。HashiCorp Vault 統合によるバージョン管理付き KV シークレットストアを提供し、Kafka 通知、SPIFFE 認証、監査ログに対応する。Rust での実装を定義する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | secrets/read |
| sys_operator 以上 | secrets/write |
| sys_admin のみ | secrets/admin |


system tier の Vault Server は以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| シークレット作成 | KV パス指定でシークレットを作成する（バージョン 1） |
| シークレット取得 | パス指定でシークレットを取得する（バージョン指定可能） |
| シークレット更新 | 既存シークレットを更新する（バージョンが自動インクリメントされる） |
| シークレット削除 | パス指定でシークレットを削除する |
| HashiCorp Vault 連携 | KV v2 / Dynamic Secrets 連携 |
| シークレットローテーション | 自動・手動によるシークレットローテーション |
| Kafka 通知 | アクセス監査は `k1s0.system.vault.access.v1`、ローテーション通知は `k1s0.system.vault.secret_rotated.v1` に配信 |
| アクセス監査ログ | シークレットアクセスの監査ログを PostgreSQL に記録 |
| ヘルスモニタリング | ヘルスチェック・レディネスチェック・Prometheus メトリクス |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Vault クライアント | vaultrs |
| キャッシュ | moka v0.12 |
| テスト | mockall 0.13, axum-test 16 |

### 配置パス

配置: `regions/system/server/rust/vault/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[Vault設計.md](../../infrastructure/security/Vault設計.md) および [サービス間認証設計.md](../../architecture/auth/サービス間認証設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ストレージ | HashiCorp Vault 連携（KV v2） |
| バージョン管理 | シークレット更新時にバージョンが自動インクリメント（KV v2 互換） |
| アクセス制御 | SPIFFE ID ベースの認可（fail-closed: ポリシー未設定時・パス不一致時は拒否） |
| キャッシュ | moka キャッシュ |
| Kafka | アクセス監査 + ローテーション通知 |
| 監査ログ | PostgreSQL に記録 |

### SPIFFE アクセス制御方針（fail-closed）

SPIFFE ミドルウェアは **fail-closed** ポリシーを採用する。

| 状態 | 動作 |
| --- | --- |
| ポリシーが1件も設定されていない | 全リクエストを 403 FORBIDDEN で拒否（`SPIFFE_NO_POLICIES`） |
| リクエストパスに一致するポリシーがない | 403 FORBIDDEN で拒否（`SPIFFE_NO_MATCHING_POLICY`） |
| パスに一致するポリシーがあり SPIFFE ID が許可リストに含まれる | 通過（200） |
| パスに一致するポリシーがあり SPIFFE ID が許可リストに含まれない | 403 FORBIDDEN で拒否（`SPIFFE_ACCESS_DENIED`） |

シークレット Vault は機密性の高い情報を扱うため、明示的にポリシーで許可されたパスのみアクセスを許可する。

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_VAULT_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/secrets` | シークレット作成 | `secrets/write` |
| GET | `/api/v1/secrets/{key}` | シークレット取得 | `secrets/read`（SPIFFE 条件も併用） |
| PUT | `/api/v1/secrets/{key}` | シークレット更新 | `secrets/write` |
| DELETE | `/api/v1/secrets/{key}` | シークレット削除 | `secrets/admin` |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/api/v1/secrets` | シークレット一覧 | `secrets/read` |
| GET | `/api/v1/secrets/{key}/metadata` | メタデータ取得 | `secrets/read` |
| POST | `/api/v1/secrets/{key}/rotate` | ローテーション | `secrets/write` |
| GET | `/api/v1/audit/logs` | 監査ログ | `secrets/read` |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/secrets

新しいシークレットを作成する。`path` でシークレットのキーパスを指定し、`data` に KV データを格納する。

**リクエスト**

```json
{
  "path": "app/db/password",
  "data": {
    "username": "db_admin",
    "password": "s3cret-v4lue"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "path": "app/db/password",
  "version": 1,
  "created_at": "2026-02-23T10:00:00.000+00:00"
}
```

#### GET /api/v1/secrets/{key}

指定パスのシークレットを取得する。バージョン未指定時は最新バージョンを返す。

**レスポンス（200 OK）**

```json
{
  "path": "app/db/password",
  "version": 1,
  "data": {
    "username": "db_admin",
    "password": "s3cret-v4lue"
  },
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T10:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_VAULT_NOT_FOUND",
    "message": "secret not found: app/db/password",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### PUT /api/v1/secrets/{key}

既存シークレットを更新する。バージョンが自動インクリメントされる。

**リクエスト**

```json
{
  "data": {
    "username": "db_admin",
    "password": "new-s3cret-v4lue"
  }
}
```

**レスポンス（200 OK）**

```json
{
  "path": "app/db/password",
  "version": 2,
  "created_at": "2026-02-23T11:00:00.000+00:00"
}
```

#### DELETE /api/v1/secrets/{key}

指定パスのシークレットを削除する。

**レスポンス（204 No Content）**

レスポンスボディなし。

#### GET /api/v1/secrets

シークレットのパス一覧を返す。

**クエリパラメータ**

| パラメータ | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `prefix` | string | `""` | パスプレフィックスでフィルタ |

**レスポンス（200 OK）**

```json
{
  "secrets": ["app/db/password", "app/api/key"]
}
```

#### GET /api/v1/secrets/{key}/metadata

指定パスのシークレットのメタデータのみを返す（シークレット値は含まない）。

**レスポンス（200 OK）**

```json
{
  "path": "app/db/password",
  "version": 3,
  "version_count": 3,
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "updated_at": "2026-02-23T12:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": "secret not found: app/db/password"
}
```

#### POST /api/v1/secrets/{key}/rotate

指定パスのシークレットをローテーションする。新しいシークレットデータをボディに渡すと、バージョンがインクリメントされる。

**リクエスト**

```json
{
  "username": "db_admin",
  "password": "rotated-s3cret-v4lue"
}
```

**レスポンス（200 OK）**

```json
{
  "path": "app/db/password",
  "new_version": 2,
  "rotated": true
}
```

#### GET /api/v1/audit/logs

シークレットアクセスの監査ログ一覧を返す。

**クエリパラメータ**

| パラメータ | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `offset` | u32 | `0` | ページオフセット |
| `limit` | u32 | `20` | 取得件数上限 |

**レスポンス（200 OK）**

```json
{
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "key_path": "app/db/password",
      "action": "read",
      "actor_id": "spiffe://cluster.local/ns/default/sa/app",
      "ip_address": "10.0.0.1",
      "success": true,
      "error_msg": null,
      "created_at": "2026-02-23T10:00:00.000+00:00"
    }
  ]
}
```

アクション種別: `read` / `write` / `delete` / `list`

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_VAULT_NOT_FOUND` | 404 | 指定されたシークレットが見つからない |
| `SYS_VAULT_ACCESS_DENIED` | 403 | アクセスが拒否された |
| `SYS_VAULT_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_VAULT_UPSTREAM_ERROR` | 502 | HashiCorp Vault への接続・クエリエラー |
| `SYS_VAULT_CACHE_ERROR` | 500 | キャッシュ操作エラー |
| `SYS_VAULT_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

`api/proto/k1s0/system/vault/v1/vault.proto` に定義。Get/Set/Delete/List に加えて、
ローテーション・メタデータ取得・監査ログ取得の RPC を提供する。

```protobuf
// k1s0 Vault シークレット管理サービス gRPC 定義。
// シークレットの取得・設定・削除・一覧取得を提供する。
syntax = "proto3";

package k1s0.system.vault.v1;

option go_package = "github.com/k1s0-platform/system-proto-go/vault/v1;vaultv1";

import "k1s0/system/common/v1/types.proto";

service VaultService {
  rpc GetSecret(GetSecretRequest) returns (GetSecretResponse);
  rpc SetSecret(SetSecretRequest) returns (SetSecretResponse);
  rpc RotateSecret(RotateSecretRequest) returns (RotateSecretResponse);
  rpc DeleteSecret(DeleteSecretRequest) returns (DeleteSecretResponse);
  rpc GetSecretMetadata(GetSecretMetadataRequest) returns (GetSecretMetadataResponse);
  rpc ListSecrets(ListSecretsRequest) returns (ListSecretsResponse);
  rpc ListAuditLogs(ListAuditLogsRequest) returns (ListAuditLogsResponse);
}

message GetSecretRequest {
  string path = 1;
  int64 version = 2;
}

message GetSecretResponse {
  map<string, string> data = 1;
  int64 version = 2;
  k1s0.system.common.v1.Timestamp created_at = 3;
  k1s0.system.common.v1.Timestamp updated_at = 4;
  string path = 5;
}

message SetSecretRequest {
  string path = 1;
  map<string, string> data = 2;
}

message SetSecretResponse {
  int64 version = 1;
  k1s0.system.common.v1.Timestamp created_at = 2;
  string path = 3;
}

message RotateSecretRequest {
  string path = 1;
  map<string, string> data = 2;
}

message RotateSecretResponse {
  string path = 1;
  int64 new_version = 2;
  bool rotated = 3;
}

message DeleteSecretRequest {
  string path = 1;
  repeated int64 versions = 2;
}

message DeleteSecretResponse {
  bool success = 1;
}

message GetSecretMetadataRequest {
  string path = 1;
}

message GetSecretMetadataResponse {
  string path = 1;
  int64 current_version = 2;
  int32 version_count = 3;
  k1s0.system.common.v1.Timestamp created_at = 4;
  k1s0.system.common.v1.Timestamp updated_at = 5;
}

message ListSecretsRequest {
  string prefix = 1;
}

message ListSecretsResponse {
  repeated string keys = 1;
}

message ListAuditLogsRequest {
  int32 offset = 1;
  int32 limit = 2;
}

message ListAuditLogsResponse {
  repeated AuditLogEntry logs = 1;
}

message AuditLogEntry {
  string id = 1;
  string key_path = 2;
  string action = 3;
  string actor_id = 4;
  string ip_address = 5;
  bool success = 6;
  optional string error_msg = 7;
  k1s0.system.common.v1.Timestamp created_at = 8;
}
```

---

## キャッシュ設計

moka を使用した TTL ベースのインメモリキャッシュにより、Vault への問い合わせ頻度を削減する。

| 設定項目 | 値 |
| --- | --- |
| キャッシュライブラリ | moka v0.12（非同期対応） |
| TTL 計算 | リース期限の 80%（例: リース 1h -> TTL 48min） |
| 最大エントリ数 | 10,000 |
| エビクションポリシー | TTL 期限切れ + LRU |
| キャッシュキー | `{path}:{version}` |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Secret`, `SecretVersion`, `SecretValue`, `SecretAccessLog`, `SpiffeAccessPolicy` | エンティティ定義 |
| domain/repository | `SecretStore`（trait）, `AccessLogRepository`（trait） | リポジトリトレイト |
| usecase | `GetSecretUseCase`, `SetSecretUseCase`, `DeleteSecretUseCase`, `ListSecretsUseCase`, `ListAuditLogsUseCase` | ユースケース |
| adapter/handler | `vault_handler.rs`（REST）, `health.rs` | axum REST ハンドラー |
| adapter/grpc | `VaultGrpcService`, `VaultServiceTonic` | tonic gRPC ハンドラー |
| adapter/gateway | `VaultClient` | HashiCorp Vault クライアント（vaultrs 経由） |
| infrastructure/persistence | `PostgresAccessLogRepository` | PostgreSQL 監査ログリポジトリ実装 |
| infrastructure/cache | `SecretCacheService` | moka キャッシュ実装 |
| infrastructure/messaging | `VaultKafkaProducer` | Kafka プロデューサー（ローテーション通知） |

### ドメインモデル

#### Secret

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `path` | String | シークレットパス（キー） |
| `current_version` | i64 | 現在のバージョン番号 |
| `versions` | Vec\<SecretVersion\> | 全バージョンのリスト |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

**メソッド:**
- `new(path, data)` -- 初期バージョン（version=1）で作成
- `get_version(version)` -- 指定バージョン（None 時は最新）を取得。destroyed 済みは None を返す
- `update(data)` -- 新バージョンを追加し、current_version をインクリメント

#### SecretVersion

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `version` | i64 | バージョン番号 |
| `value` | SecretValue | シークレット値 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `destroyed` | bool | 破棄済みフラグ |

#### SecretValue

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `data` | HashMap\<String, String\> | シークレットデータ（key-value ペア） |

#### SecretAccessLog

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | Uuid | ログエントリ ID |
| `path` | String | アクセス対象のシークレットパス |
| `action` | AccessAction | アクション種別（Read / Write / Delete / List） |
| `subject` | Option\<String\> | アクセス主体（SPIFFE ID） |
| `tenant_id` | Option\<String\> | テナント ID |
| `ip_address` | Option\<String\> | クライアント IP |
| `trace_id` | Option\<String\> | OTel トレース ID |
| `success` | bool | 成功フラグ |
| `error_msg` | Option\<String\> | エラーメッセージ（失敗時） |
| `created_at` | DateTime\<Utc\> | 記録日時 |

#### SpiffeAccessPolicy

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | Uuid | ポリシー ID |
| `secret_path_pattern` | String | 対象シークレットパスの glob パターン（`*`/`**`） |
| `allowed_spiffe_ids` | Vec\<String\> | アクセスを許可する SPIFFE ID 一覧 |
| `created_at` | DateTime\<Utc\> | 作成日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (vault_handler.rs)          │   │
                    │  │  healthz / readyz                        │   │
                    │  │  create_secret / get_secret              │   │
                    │  │  update_secret / delete_secret           │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (vault_grpc.rs)             │   │
                    │  │  VaultGrpcService                        │   │
                    │  │  get_secret / set_secret                 │   │
                    │  │  delete_secret / list_secrets            │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  GetSecretUseCase / SetSecretUseCase /          │
                    │  DeleteSecretUseCase / ListSecretsUseCase       │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────────┐         ┌──────────▼──────────────────┐   │
    │  domain/entity      │         │ domain/repository           │   │
    │  Secret,            │         │ SecretStore (trait)          │   │
    │  SecretVersion,     │         │ AccessLogRepository (trait)  │   │
    │  SecretValue,       │         └─────────────────────────────┘   │
    │  SecretAccessLog    │                                           │
    └─────────────────────┘                                           │
                                                                      │
                    ┌─────────────────────────────────────────────────┘
                    │             infrastructure 層
                    │  ┌────────────────────────────────────────────┐ │
                    │  │ VaultClient (vaultrs)                      │ │
                    │  │ PostgresAccessLogRepository                │ │
                    │  │ SecretCacheService (moka)                  │ │
                    │  │ VaultKafkaProducer (rdkafka)               │ │
                    │  ├────────────────────────────────────────────┤ │
                    │  │ Config Loader (serde_yaml)                 │ │
                    │  └────────────────────────────────────────────┘ │
                    └─────────────────────────────────────────────────┘
```

---

## 設定ファイル

### config.yaml
> ※ dev環境では省略可能なセクションがあります。


```yaml
app:
  name: k1s0-vault-server
  version: "0.1.0"
  environment: dev
server:
  host: "0.0.0.0"
  port: 8090
  grpc_port: 50051

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"
  issuer: "https://auth.k1s0.example.com/realms/system"
  audience: "k1s0-system"
  jwks_cache_ttl_secs: 3600

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""
  ssl_mode: "require"          # 開発環境では "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "vault-server.default"
  security_protocol: "PLAINTEXT"
  sasl:
    mechanism: ""
    username: ""
    password: ""
  topics:
    publish:
      - "k1s0.system.vault.access.v1"
      - "k1s0.system.vault.secret_rotated.v1"
    subscribe: []
```

---

## デプロイ

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。vault-server 固有の values は以下の通り。

```yaml
# values-vault.yaml（infra/helm/services/system/vault/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/vault-server
  tag: ""

replicaCount: 2

container:
  port: 8090
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
```

---

## 詳細設計ドキュメント

- [system-vault-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-vault-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [Vault設計.md](../../infrastructure/security/Vault設計.md) -- HashiCorp Vault 全体設計
- [サービス間認証設計.md](../../architecture/auth/サービス間認証設計.md) -- SPIFFE/SPIRE によるサービス間認証
- [REST-API設計.md](../../architecture/api/REST-API設計.md) -- D-007 統一エラーレスポンス

## Doc Sync (2026-03-03)

### gRPC Canonical RPCs (proto)
- `GetSecret`, `SetSecret`, `RotateSecret`, `DeleteSecret`
- `GetSecretMetadata`, `ListSecrets`, `ListAuditLogs`

### Message/Field Corrections
- Canonical messages include `RotateSecretRequest/Response`, `GetSecretMetadataRequest/Response`, `ListAuditLogsRequest/Response`, `AuditLogEntry`.
- `GetSecretResponse.updated_at` is present.


### 2026-03-03 追補
- GET /api/v1/secrets/{key}/metadata の 404 は標準 ErrorResponse（error.code/message/request_id/details）で返却する。
- RBAC は secrets/read, secrets/write, secrets/admin の resource/action 併記を正とする。
---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
