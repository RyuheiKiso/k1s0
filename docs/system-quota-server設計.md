# system-quota-server 設計

system tier のAPIクォータ管理サーバー設計を定義する。テナント・ユーザー・APIキーごとの日次/月次クォータを管理し、Redis を活用した低レイテンシな使用量照会と超過検知を提供する。Kafka トピック `k1s0.system.quota.exceeded.v1` で超過イベントを発行する。Rust での実装を定義する。

## 概要

system tier のAPIクォータ管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| クォータポリシー管理 | テナント・ユーザー・APIキーごとの日次/月次クォータ定義の CRUD |
| 使用量照会 | Redis を活用した低レイテンシな残量照会 |
| 使用量インクリメント | アトミックな使用量加算と超過判定 |
| クォータ超過検知・通知 | 超過時に Kafka イベント発行・notification-server との連携 |
| 使用量リセット | 定期リセット（日次・月次）および手動リセット |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| DB アクセス | sqlx v0.8 |
| キャッシュ | deadpool-redis（Redis 接続プール） |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/quota/` |

---

## 設計方針

[認証認可設計.md](認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| 使用量カウンター | Redis の INCR + EXPIRE によるアトミック加算。キー形式: `quota:{policy_id}:{period}` |
| 永続化 | PostgreSQL の `quota` スキーマ（quota_policies, quota_usages テーブル）に使用量を記録 |
| 超過検知 | インクリメント時に閾値と比較し、超過時は Kafka `k1s0.system.quota.exceeded.v1` を発行 |
| 通知連携 | Kafka 経由で notification-server に通知依頼を送信 |
| リセット | 日次・月次リセットは tokio スケジューラーで自動実行。手動リセットは REST API で提供 |
| 認可 | 参照は `sys_auditor`、操作は `sys_operator`、削除・リセットは `sys_admin` |
| ポート | ホスト側 8097（内部 8080） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_QUOTA_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/quotas` | クォータポリシー一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/quotas` | クォータポリシー作成 | `sys_operator` 以上 |
| GET | `/api/v1/quotas/:id` | クォータポリシー取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/quotas/:id` | クォータポリシー更新 | `sys_operator` 以上 |
| POST | `/api/v1/quotas/:id/check` | クォータチェック（残量判定） | `sys_operator` 以上 |
| DELETE | `/api/v1/quotas/:id` | クォータポリシー削除 | `sys_admin` のみ |
| GET | `/api/v1/quotas/:id/usage` | 使用量照会 | `sys_auditor` 以上 |
| POST | `/api/v1/quotas/:id/usage/increment` | 使用量インクリメント | `sys_operator` 以上 |
| POST | `/api/v1/quotas/:id/usage/reset` | 使用量リセット | `sys_admin` のみ |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/quotas

クォータポリシー一覧をページネーション付きで取得する。`subject_type` クエリパラメータでフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `subject_type` | string | No | - | 対象種別でフィルタ（tenant/user/api_key） |
| `subject_id` | string | No | - | 対象 ID でフィルタ |
| `enabled_only` | bool | No | false | 有効なポリシーのみ取得 |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "quotas": [
    {
      "id": "quota_01JABCDEF1234567890",
      "name": "スタンダードプランAPIクォータ",
      "subject_type": "tenant",
      "subject_id": "tenant-abc",
      "limit": 10000,
      "period": "daily",
      "enabled": true,
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 15,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

#### POST /api/v1/quotas

新しいクォータポリシーを作成する。`subject_type` は `tenant` / `user` / `api_key` のいずれかを指定する。`period` は `daily` / `monthly` のいずれかを指定する。

**リクエスト**

```json
{
  "name": "スタンダードプランAPIクォータ",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "limit": 10000,
  "period": "daily",
  "enabled": true,
  "alert_threshold_percent": 80
}
```

**レスポンス（201 Created）**

```json
{
  "id": "quota_01JABCDEF1234567890",
  "name": "スタンダードプランAPIクォータ",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "limit": 10000,
  "period": "daily",
  "enabled": true,
  "alert_threshold_percent": 80,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_QUOTA_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "subject_type", "message": "must be one of: tenant, user, api_key"},
      {"field": "limit", "message": "limit must be greater than 0"}
    ]
  }
}
```

#### GET /api/v1/quotas/:id/usage

指定クォータポリシーの現在の使用量を照会する。Redis から低レイテンシで取得し、残量・使用率を返す。

**レスポンス（200 OK）**

```json
{
  "quota_id": "quota_01JABCDEF1234567890",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "period": "daily",
  "limit": 10000,
  "used": 7523,
  "remaining": 2477,
  "usage_percent": 75.23,
  "exceeded": false,
  "period_start": "2026-02-23T00:00:00.000+00:00",
  "period_end": "2026-02-23T23:59:59.999+00:00",
  "reset_at": "2026-02-24T00:00:00.000+00:00",
  "retrieved_at": "2026-02-23T14:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_QUOTA_NOT_FOUND",
    "message": "quota policy not found: quota_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/quotas/:id/usage/increment

使用量をアトミックに加算する。加算後に超過判定を行い、閾値超過時は Kafka イベントを発行する。

**リクエスト**

```json
{
  "amount": 1,
  "request_id": "req_abc123def456"
}
```

**レスポンス（200 OK）**

```json
{
  "quota_id": "quota_01JABCDEF1234567890",
  "used": 7524,
  "remaining": 2476,
  "usage_percent": 75.24,
  "exceeded": false,
  "allowed": true
}
```

**レスポンス（429 Too Many Requests）**

```json
{
  "error": {
    "code": "SYS_QUOTA_EXCEEDED",
    "message": "quota exceeded for tenant-abc: 10000/10000 (daily)",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "quota_id", "message": "quota_01JABCDEF1234567890"},
      {"field": "reset_at", "message": "2026-02-24T00:00:00.000+00:00"}
    ]
  }
}
```

#### POST /api/v1/quotas/:id/usage/reset

使用量を手動でリセットする。`sys_admin` のみ実行可能。リセット理由の記録が必須。

**リクエスト**

```json
{
  "reason": "プラン変更に伴うリセット"
}
```

**レスポンス（200 OK）**

```json
{
  "quota_id": "quota_01JABCDEF1234567890",
  "used": 0,
  "reset_at": "2026-02-23T15:00:00.000+00:00",
  "reset_by": "admin@example.com"
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_QUOTA_NOT_FOUND` | 404 | 指定されたクォータポリシーが見つからない |
| `SYS_QUOTA_ALREADY_EXISTS` | 409 | 同一 subject に対するクォータポリシーが既に存在する |
| `SYS_QUOTA_EXCEEDED` | 429 | クォータ上限を超過している |
| `SYS_QUOTA_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_QUOTA_REDIS_ERROR` | 502 | Redis への接続・操作エラー |
| `SYS_QUOTA_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.quota.v1;

service QuotaService {
  rpc CheckAndIncrement(CheckAndIncrementRequest) returns (CheckAndIncrementResponse);
  rpc GetUsage(GetUsageRequest) returns (GetUsageResponse);
}

message CheckAndIncrementRequest {
  string quota_id = 1;
  uint64 amount = 2;
  string request_id = 3;
}

message CheckAndIncrementResponse {
  string quota_id = 1;
  uint64 used = 2;
  uint64 remaining = 3;
  double usage_percent = 4;
  bool exceeded = 5;
  bool allowed = 6;
}

message GetUsageRequest {
  string quota_id = 1;
}

message GetUsageResponse {
  QuotaUsage usage = 1;
}

message QuotaUsage {
  string quota_id = 1;
  string subject_type = 2;
  string subject_id = 3;
  string period = 4;
  uint64 limit = 5;
  uint64 used = 6;
  uint64 remaining = 7;
  double usage_percent = 8;
  bool exceeded = 9;
  string period_start = 10;
  string period_end = 11;
  string reset_at = 12;
  string retrieved_at = 13;
}
```

---

## Kafka メッセージング設計

### クォータ超過イベント

クォータ超過を検知した際に `k1s0.system.quota.exceeded.v1` トピックへ発行する。notification-server はこのイベントを Consumer して管理者への通知を行う。

**メッセージフォーマット**

```json
{
  "event_type": "QUOTA_EXCEEDED",
  "quota_id": "quota_01JABCDEF1234567890",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "period": "daily",
  "limit": 10000,
  "used": 10001,
  "exceeded_at": "2026-02-23T14:30:00.000+00:00",
  "reset_at": "2026-02-24T00:00:00.000+00:00"
}
```

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.quota.exceeded.v1` |
| キー | quota_id |
| パーティション戦略 | subject_id によるハッシュ分散 |

### アラート閾値通知イベント

使用率が `alert_threshold_percent` に達した時点でも通知イベントを発行する（超過前の早期警告）。

**メッセージフォーマット**

```json
{
  "event_type": "QUOTA_THRESHOLD_REACHED",
  "quota_id": "quota_01JABCDEF1234567890",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "period": "daily",
  "limit": 10000,
  "used": 8002,
  "usage_percent": 80.02,
  "alert_threshold_percent": 80,
  "reached_at": "2026-02-23T12:00:00.000+00:00"
}
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
adapter（REST ハンドラー・gRPC ハンドラー）
  ^
infrastructure（DB接続・Redis・Kafka Producer・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `QuotaPolicy`, `QuotaUsage` | エンティティ定義 |
| domain/repository | `QuotaPolicyRepository`, `QuotaUsageRepository`, `QuotaCounterRepository` | リポジトリトレイト |
| domain/service | `QuotaDomainService` | 超過判定・アラート閾値判定ロジック |
| usecase | `CreateQuotaPolicyUsecase`, `UpdateQuotaPolicyUsecase`, `DeleteQuotaPolicyUsecase`, `GetQuotaPolicyUsecase`, `ListQuotaPoliciesUsecase`, `GetQuotaUsageUsecase`, `IncrementQuotaUsageUsecase`, `ResetQuotaUsageUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `QuotaPolicyPostgresRepository`, `QuotaUsagePostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/cache | `QuotaRedisCounterRepository` | Redis アトミックカウンター実装 |
| infrastructure/messaging | `QuotaExceededKafkaProducer` | Kafka プロデューサー（超過イベント発行） |

### ドメインモデル

#### QuotaPolicy

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | クォータポリシーの一意識別子 |
| `name` | String | ポリシーの表示名 |
| `subject_type` | String | 対象種別（tenant / user / api_key） |
| `subject_id` | String | 対象の ID |
| `limit` | u64 | クォータ上限値 |
| `period` | String | 集計期間（daily / monthly） |
| `enabled` | bool | ポリシーの有効/無効 |
| `alert_threshold_percent` | Option\<u8\> | アラート発行する使用率閾値（0-100） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### QuotaUsage

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `quota_id` | String | クォータポリシー ID |
| `subject_type` | String | 対象種別 |
| `subject_id` | String | 対象の ID |
| `period` | String | 集計期間 |
| `limit` | u64 | クォータ上限値 |
| `used` | u64 | 現在の使用量 |
| `remaining` | u64 | 残量 |
| `usage_percent` | f64 | 使用率 |
| `exceeded` | bool | 超過フラグ |
| `period_start` | DateTime\<Utc\> | 集計期間の開始日時 |
| `period_end` | DateTime\<Utc\> | 集計期間の終了日時 |
| `reset_at` | DateTime\<Utc\> | 次回リセット日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (quota_handler.rs)          │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_quotas / create_quota /            │   │
                    │  │  get_quota / update_quota / delete_quota │   │
                    │  │  get_usage / increment_usage / reset     │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (quota_grpc.rs)             │   │
                    │  │  CheckAndIncrement / GetUsage            │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateQuotaPolicyUsecase /                     │
                    │  UpdateQuotaPolicyUsecase /                     │
                    │  DeleteQuotaPolicyUsecase /                     │
                    │  GetQuotaPolicyUsecase /                        │
                    │  ListQuotaPoliciesUsecase /                     │
                    │  GetQuotaUsageUsecase /                         │
                    │  IncrementQuotaUsageUsecase /                   │
                    │  ResetQuotaUsageUsecase                         │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  QuotaPolicy,   │              │ QuotaPolicyRepository      │   │
    │  QuotaUsage     │              │ QuotaUsageRepository       │   │
    └────────────────┘              │ QuotaCounterRepository     │   │
              │                     │ (trait)                    │   │
              │  ┌────────────────┐  └──────────┬─────────────────┘   │
              └──▶ domain/service │             │                     │
                 │ QuotaDomain   │             │                     │
                 │ Service       │             │                     │
                 └────────────────┘             │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ QuotaPolicy/Usage       │  │
                    │  │ Producer     │  │ PostgresRepository      │  │
                    │  │ (exceeded)   │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ QuotaRedisCounter      │  │
                    │  │ Config       │  │ Repository (deadpool)   │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "quota"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

database:
  url: "postgresql://app:@postgres.k1s0-system.svc.cluster.local:5432/k1s0_system"
  schema: "quota"
  max_connections: 10
  min_connections: 2
  connect_timeout_seconds: 5

redis:
  url: "redis://redis.k1s0-system.svc.cluster.local:6379"
  pool_size: 10
  key_prefix: "quota:"
  connect_timeout_seconds: 3

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic_exceeded: "k1s0.system.quota.exceeded.v1"
  topic_threshold: "k1s0.system.quota.threshold.reached.v1"

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"

quota:
  reset_schedule:
    daily: "0 0 * * *"
    monthly: "0 0 1 * *"
```

---

## デプロイ

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。quota 固有の values は以下の通り。

```yaml
# values-quota.yaml（infra/helm/services/system/quota/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/quota
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
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/quota/database"
      key: "password"
      mountPath: "/vault/secrets/database-password"
    - path: "secret/data/k1s0/system/quota/redis"
      key: "password"
      mountPath: "/vault/secrets/redis-password"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/quota/database` |
| Redis パスワード | `secret/data/k1s0/system/quota/redis` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-quota-server-実装設計.md](system-quota-server-実装設計.md) -- 実装設計の詳細
- [system-quota-server-デプロイ設計.md](system-quota-server-デプロイ設計.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

- [RBAC設計.md](RBAC設計.md) -- RBAC ロールモデル
- [認証認可設計.md](認証認可設計.md) -- RBAC 認可モデル
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [メッセージング設計.md](メッセージング設計.md) -- Kafka メッセージング設計
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](コーディング規約.md) -- コーディング規約
- [system-server設計.md](system-server設計.md) -- system tier サーバー一覧
- [system-server-実装設計.md](system-server-実装設計.md) -- system tier 実装設計
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
- [system-notification-server設計.md](system-notification-server設計.md) -- 通知サーバー（超過時の通知連携）
