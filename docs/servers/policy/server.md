# system-policy-server 設計

OPA 連携の動的ポリシー評価サーバー。Rego ポリシー管理・バンドル管理・評価キャッシュを提供。

## 概要

system tier のポリシー評価サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ポリシー定義管理 | Rego ポリシーの作成・更新・削除・一覧取得 |
| ポリシー評価 | 入力データに対する allow/deny 判定（OPA HTTP API 経由） |
| ポリシーバンドル管理 | 複数ポリシーのグループ管理・バンドル単位でのデプロイ |
| ポリシー変更通知 | ポリシー変更時に Kafka `k1s0.system.policy.updated.v1` で通知 |
| 評価キャッシュ | 評価結果を moka で TTL キャッシュし、OPA 呼び出しを削減 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| キャッシュ | moka v0.12 |
| OPA クライアント | opa-client（OPA HTTP API 呼び出し） |

### 配置パス

配置: `regions/system/server/rust/policy/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

### gRPC ポート

proto ファイルおよびサーバー実装のデフォルト: **50051**（config.yaml で上書き可能）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) および [RBAC設計.md](../../architecture/auth/RBAC設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ポリシーエンジン | OPA（Open Policy Agent）。opa-client クレート経由で OPA HTTP API を呼び出す |
| ポリシー言語 | Rego。ポリシー本文は PostgreSQL の `policy.policies` テーブルで管理 |
| 評価フロー | REST/gRPC リクエスト → Rust サーバー → OPA HTTP API（/v1/data/{package_path}） → 評価結果返却。OPA 未設定時は policy.enabled フラグでフォールバック評価 |
| キャッシュ | moka で評価結果を TTL 30 秒キャッシュ。Kafka 通知受信時にキャッシュ無効化 |
| DB スキーマ | PostgreSQL の `policy` スキーマ（policies, policy_bundles テーブル） |
| Kafka | ポリシー変更時に `k1s0.system.policy.updated.v1` トピックへ変更通知を送信 |
| ポート | ホスト側 8096（内部 8080）、gRPC 50051 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_POLICY_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/policies` | ポリシー一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/policies/:id` | ポリシー詳細取得 | `sys_auditor` 以上 |
| POST | `/api/v1/policies` | ポリシー作成 | `sys_admin` のみ |
| PUT | `/api/v1/policies/:id` | ポリシー更新 | `sys_admin` のみ |
| DELETE | `/api/v1/policies/:id` | ポリシー削除 | `sys_admin` のみ |
| POST | `/api/v1/policies/:id/evaluate` | ポリシー評価（ポリシー ID 指定） | `sys_operator` 以上 |
| GET | `/api/v1/bundles` | バンドル一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/bundles` | バンドル作成 | `sys_admin` のみ |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/policies

登録済みポリシーの全一覧を取得する。

**レスポンス例（200 OK）**

```json
{
  "policies": [
    {
      "id": "policy-001",
      "name": "k1s0-tenant-access",
      "description": "テナントへのアクセス制御ポリシー",
      "package_path": "k1s0.system.tenant",
      "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
      "bundle_id": "bundle-001",
      "enabled": true,
      "version": 3,
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ]
}
```

> 現在の実装ではページネーションは未実装。全件返却となる。`bundle_id`/`enabled_only` クエリフィルタも未実装。

#### GET /api/v1/policies/:id

ID 指定でポリシーの詳細を取得する。

**レスポンス例（200 OK）**

```json
{
  "id": "policy-001",
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー",
  "package_path": "k1s0.system.tenant",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
  "bundle_id": "bundle-001",
  "enabled": true,
  "version": 3,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_POLICY_NOT_FOUND",
    "message": "policy not found: policy-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/policies

新しい Rego ポリシーを作成する。作成時に OPA への同期も行い、Kafka 変更通知を送信する。

**リクエスト例**

```json
{
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
  "package_path": "k1s0.system.tenant",
  "bundle_id": "bundle-001"
}
```

> `package_path` は省略可能。`bundle_id` は省略可能。`enabled` はデフォルト `true`（リクエストに含める必要なし）。

**レスポンス例（201 Created）**

```json
{
  "id": "policy-001",
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー",
  "package_path": "k1s0.system.tenant",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
  "bundle_id": "bundle-001",
  "enabled": true,
  "version": 1,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス例（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_POLICY_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "rego_content", "message": "invalid Rego syntax: unexpected token at line 3"},
      {"field": "package_path", "message": "package_path is required and must be non-empty"}
    ]
  }
}
```

#### PUT /api/v1/policies/:id

既存のポリシーを更新する。更新時にバージョンを自動インクリメントし、OPA への同期、Kafka 変更通知を行う。キャッシュは即座に無効化される。

**リクエスト例**

```json
{
  "description": "テナントへのアクセス制御ポリシー（v2 - operatorも許可）",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}\n\nallow {\n  input.role == \"sys_operator\"\n}",
  "enabled": true
}
```

**レスポンス例（200 OK）**

```json
{
  "id": "policy-001",
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー（v2 - operatorも許可）",
  "package_path": "k1s0.system.tenant",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}\n\nallow {\n  input.role == \"sys_operator\"\n}",
  "bundle_id": "bundle-001",
  "enabled": true,
  "version": 4,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T15:00:00.000+00:00"
}
```

#### DELETE /api/v1/policies/:id

ポリシーを削除する。削除時に OPA からもポリシーを削除し、Kafka 変更通知を送信する。

**レスポンス例（200 OK）**

```json
{
  "success": true,
  "message": "policy policy-001 deleted"
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_POLICY_NOT_FOUND",
    "message": "policy not found: policy-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/policies/:id/evaluate

指定ポリシーに対して入力データを評価し、allow/deny を返す。評価結果は moka キャッシュに TTL 30 秒で保存される。

**リクエスト例**

```json
{
  "input": {
    "role": "sys_operator",
    "action": "read",
    "resource": "tenant",
    "tenant_id": "tenant-abc"
  }
}
```

> ポリシー ID は URL パス（`:id`）で指定する。`package_path` はサーバー内でポリシーから自動解決される。

**レスポンス例（200 OK -- 許可）**

```json
{
  "allowed": true,
  "reason": "OPA evaluation: allowed"
}
```

**レスポンス例（200 OK -- 拒否）**

```json
{
  "allowed": false,
  "reason": "OPA evaluation: denied"
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_POLICY_NOT_FOUND",
    "message": "policy not found for package: k1s0.system.tenant",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/bundles

登録済みバンドルの一覧を取得する。

**レスポンス例（200 OK）**

```json
{
  "bundles": [
    {
      "id": "bundle-001",
      "name": "k1s0-system-policies",
      "policy_ids": ["policy-001", "policy-002", "policy-003"],
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ]
}
```

#### POST /api/v1/bundles

複数のポリシーをグループ化したバンドルを作成する。

**リクエスト例**

```json
{
  "name": "k1s0-system-policies",
  "policy_ids": ["policy-001", "policy-002", "policy-003"]
}
```

**レスポンス例（201 Created）**

```json
{
  "id": "bundle-001",
  "name": "k1s0-system-policies",
  "policy_ids": ["policy-001", "policy-002", "policy-003"],
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス例（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_POLICY_INVALID_ID",
    "message": "invalid policy_id format"
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_POLICY_NOT_FOUND` | 404 | 指定されたポリシーが見つからない |
| `SYS_POLICY_BUNDLE_NOT_FOUND` | 404 | 指定されたバンドルが見つからない |
| `SYS_POLICY_ALREADY_EXISTS` | 409 | 同一名のポリシーが既に存在する |
| `SYS_POLICY_VALIDATION_ERROR` | 400 | リクエストまたは Rego 構文のバリデーションエラー |
| `SYS_POLICY_OPA_ERROR` | 502 | OPA への接続・評価エラー |
| `SYS_POLICY_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

gRPC ポート: **50051**


```protobuf
syntax = "proto3";
package k1s0.system.policy.v1;

import "k1s0/system/common/v1/types.proto";

service PolicyService {
  rpc EvaluatePolicy(EvaluatePolicyRequest) returns (EvaluatePolicyResponse);
  rpc GetPolicy(GetPolicyRequest) returns (GetPolicyResponse);
}

message EvaluatePolicyRequest {
  string package_path = 1;
  bytes input_json = 2;
}

message EvaluatePolicyResponse {
  bool allowed = 1;
  string package_path = 2;
  string decision_id = 3;
  bool cached = 4;
}

message GetPolicyRequest {
  string id = 1;
}

message GetPolicyResponse {
  Policy policy = 1;
}

message Policy {
  string id = 1;
  string name = 2;
  string description = 3;
  string package_path = 4;
  string rego_content = 5;
  string bundle_id = 6;
  bool enabled = 7;
  uint32 version = 8;
  k1s0.system.common.v1.Timestamp created_at = 9;
  k1s0.system.common.v1.Timestamp updated_at = 10;
}
```

---

## Kafka メッセージング設計

### ポリシー変更通知

ポリシーの作成・更新・削除時に以下のメッセージを Kafka トピック `k1s0.system.policy.updated.v1` に送信する。

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.policy.updated.v1` |
| acks | `all` |
| message.timeout.ms | `5000` |
| キー | ポリシー ID（例: `policy-001`） |

**メッセージ例**

```json
{
  "event_type": "POLICY_UPDATED",
  "policy_id": "policy-001",
  "package_path": "k1s0.system.tenant",
  "operation": "UPDATE",
  "version": 4,
  "timestamp": "2026-02-20T15:00:00.000+00:00",
  "actor_user_id": "admin-001"
}
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Policy`, `PolicyBundle`, `PolicyEvaluation` | エンティティ定義 |
| domain/repository | `PolicyRepository`, `PolicyBundleRepository` | リポジトリトレイト |
| domain/service | `PolicyDomainService` | Rego 構文バリデーション・評価結果キャッシュキー生成 |
| usecase | `GetPolicyUseCase`, `CreatePolicyUseCase`, `UpdatePolicyUseCase`, `DeletePolicyUseCase`, `EvaluatePolicyUseCase`, `CreateBundleUseCase`, `ListBundlesUseCase` | ユースケース（list_policies は repository 直呼び出し） |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `PolicyPostgresRepository`, `PolicyBundlePostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/cache | `PolicyEvalCacheService` | moka キャッシュ実装（評価結果キャッシュ） |
| infrastructure/opa | `OpaHttpClient` | OPA HTTP API 呼び出し実装 |
| infrastructure/messaging | `PolicyChangeKafkaProducer` | Kafka プロデューサー（ポリシー変更通知） |

### ドメインモデル

#### Policy

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ポリシーの一意識別子 |
| `name` | String | ポリシー名（例: `k1s0-tenant-access`） |
| `description` | String | ポリシーの説明 |
| `package_path` | String | Rego パッケージパス（例: `k1s0.system.tenant`） |
| `rego_content` | String | Rego ポリシー本文 |
| `bundle_id` | Option\<UUID\> | 所属バンドル ID |
| `enabled` | bool | ポリシーの有効/無効 |
| `version` | u32 | ポリシーのバージョン（更新ごとにインクリメント） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### PolicyBundle

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | バンドルの一意識別子 |
| `name` | String | バンドル名（例: `k1s0-system-policies`） |
| `policy_ids` | Vec\<UUID\> | 所属ポリシー ID 一覧 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### PolicyEvaluation

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `package_path` | String | 評価対象 Rego パッケージパス |
| `input` | serde_json::Value | 評価入力データ |
| `allowed` | bool | 評価結果（true: 許可 / false: 拒否） |
| `decision_id` | String | OPA 評価 ID |
| `cached` | bool | キャッシュヒットフラグ |
| `evaluated_at` | DateTime\<Utc\> | 評価日時 |

### キャッシュ戦略

| 項目 | 値 |
| --- | --- |
| キャッシュライブラリ | moka v0.12 |
| キャッシュキー | `{package_path}:{input_hash}` （入力 JSON の SHA-256 ハッシュ） |
| TTL | 30 秒 |
| 最大エントリ数 | 50,000 |
| 無効化トリガー | ポリシー更新・削除時に該当 package_path のエントリを即座に無効化 + Kafka 通知受信時 |

---

## DB スキーマ

PostgreSQL の `policy` スキーマに以下のテーブルを配置する。

```sql
CREATE SCHEMA IF NOT EXISTS policy;

CREATE TABLE policy.policy_bundles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    enabled     BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE policy.policies (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name         TEXT NOT NULL UNIQUE,
    description  TEXT NOT NULL DEFAULT '',
    package_path TEXT NOT NULL UNIQUE,
    rego_content TEXT NOT NULL,
    bundle_id    UUID REFERENCES policy.policy_bundles(id) ON DELETE SET NULL,
    enabled      BOOLEAN NOT NULL DEFAULT true,
    version      INTEGER NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_policies_bundle_id ON policy.policies(bundle_id);
CREATE INDEX idx_policies_package_path ON policy.policies(package_path);
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (policy_handler.rs)         │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_policies / get_policy              │   │
                    │  │  create_policy / update_policy           │   │
                    │  │  delete_policy / evaluate_policy         │   │
                    │  │  list_bundles / create_bundle            │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (tonic_service.rs)          │   │
                    │  │  EvaluatePolicy / GetPolicy              │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  GetPolicyUseCase / CreatePolicyUseCase /       │
                    │  UpdatePolicyUseCase / DeletePolicyUseCase /    │
                    │  EvaluatePolicyUseCase / CreateBundleUseCase /  │
                    │  ListBundlesUseCase                             │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  Policy,        │              │ PolicyRepository           │   │
    │  PolicyBundle,  │              │ PolicyBundleRepository     │   │
    │  PolicyEvaluation              │ (trait)                    │   │
    └────────────────┘              └──────────┬─────────────────┘   │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ PolicyDomain   │            │                     │
                 │ Service        │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ PolicyPostgres         │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ PolicyBundlePostgres   │  │
                    │  │ moka Cache   │  │ Repository             │  │
                    │  │ Service      │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ Database               │  │
                    │  │ OPA HTTP     │  │ Config                 │  │
                    │  │ Client       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "policy"
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

opa:
  url: "http://opa.k1s0-system.svc.cluster.local:8181"
  timeout_ms: 2000

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic: "k1s0.system.policy.updated.v1"

cache:
  max_entries: 50000
  ttl_seconds: 30
```

### Helm values

```yaml
# values-policy.yaml（infra/helm/services/system/policy/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/policy
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
    - path: "secret/data/k1s0/system/policy/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/policy/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-policy-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-policy-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC ロールモデル
- [system-server.md](../auth/server.md) -- system tier サーバー一覧
