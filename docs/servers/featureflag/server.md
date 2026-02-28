# system-featureflag-server 設計

system tier のフィーチャーフラグサーバー設計を定義する。全サービスに動的な機能制御を提供し、フラグ定義を PostgreSQL で管理する。フラグ変更時は Kafka トピック `k1s0.system.featureflag.changed.v1` で全サービスに通知する。
Rust での実装を定義する。

## 概要

system tier のフィーチャーフラグサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| フラグ定義管理 | フィーチャーフラグの作成・更新・削除・一覧取得 |
| フラグ評価 | ユーザー・テナント・属性に基づくフラグの評価（有効/無効 + バリアント選択） |
| バリアント/ルール制御 | バリアント（重み付き値）とルール（属性マッチング → バリアント選択）による柔軟な制御 |
| 変更通知 | フラグ変更時に Kafka `k1s0.system.featureflag.changed.v1` で全サービスに通知 |
| 変更監査ログ | フラグ変更を PostgreSQL に記録し、変更前後の値を保存 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| キャッシュ | moka v0.12 |

### 配置パス

配置: `regions/system/server/rust/featureflag/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| フラグ評価方式 | サーバー側評価。クライアントは gRPC/REST でフラグ値を問い合わせる |
| キャッシュ | moka で評価結果を TTL 60 秒キャッシュ。Kafka 通知受信時にキャッシュ無効化 |
| バリアント/ルール | バリアント（name/value/weight）による値の定義、ルール（attribute/operator/value → variant）による条件分岐 |
| DB | PostgreSQL の `featureflag` スキーマ |
| Kafka | オプション。未設定時は変更通知なし（REST/gRPC API は動作する） |
| 監査ログ | フラグの作成・更新・削除時に変更前後の値を PostgreSQL に記録 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_FF_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/flags` | フラグ一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/flags/:key` | フラグ詳細取得 | `sys_auditor` 以上 |
| POST | `/api/v1/flags` | フラグ作成 | `sys_operator` 以上 |
| PUT | `/api/v1/flags/:key` | フラグ更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/flags/:key` | フラグ削除 | `sys_admin` のみ |
| POST | `/api/v1/flags/:key/evaluate` | フラグ評価 | 不要（内部サービス用） |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/flags

フラグ一覧を取得する。リポジトリの `find_all()` で全件取得する。

**レスポンス（200 OK）**

```json
{
  "flags": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "flag_key": "enable-new-checkout",
      "description": "新しいチェックアウトフローを有効化する",
      "enabled": true,
      "variants": [
        { "name": "on", "value": "true", "weight": 80 },
        { "name": "off", "value": "false", "weight": 20 }
      ],
      "created_at": "2026-02-20T10:00:00+00:00",
      "updated_at": "2026-02-20T12:30:00+00:00"
    }
  ]
}
```

#### GET /api/v1/flags/:key

キー指定でフラグの詳細を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": true,
  "variants": [
    { "name": "on", "value": "true", "weight": 80 },
    { "name": "off", "value": "false", "weight": 20 }
  ],
  "created_at": "2026-02-20T10:00:00+00:00",
  "updated_at": "2026-02-20T12:30:00+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "feature flag not found: enable-new-checkout",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/flags

新しいフィーチャーフラグを作成する。フラグキーはシステム全体で一意でなければならない。

**リクエスト**

```json
{
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": false,
  "variants": [
    { "name": "on", "value": "true", "weight": 100 }
  ]
}
```

`variants` は省略可能（省略時は空リスト）。

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": false,
  "variants": [
    { "name": "on", "value": "true", "weight": 100 }
  ],
  "created_at": "2026-02-20T10:00:00+00:00",
  "updated_at": "2026-02-20T10:00:00+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_FF_ALREADY_EXISTS",
    "message": "flag already exists: enable-new-checkout"
  }
}
```

#### PUT /api/v1/flags/:key

既存のフィーチャーフラグを更新する。`enabled` と `description` のみ更新可能（部分更新）。

**リクエスト**

```json
{
  "enabled": true,
  "description": "新しいチェックアウトフローを有効化する（v2）"
}
```

全フィールド省略可能（省略時は変更なし）。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する（v2）",
  "enabled": true,
  "variants": [
    { "name": "on", "value": "true", "weight": 100 }
  ],
  "created_at": "2026-02-20T10:00:00+00:00",
  "updated_at": "2026-02-20T12:30:00+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "flag not found: enable-new-checkout"
  }
}
```

#### DELETE /api/v1/flags/:key

フィーチャーフラグを削除する。まず `GET` でフラグを取得し、存在確認後に ID で削除する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "flag enable-new-checkout deleted"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "flag not found: enable-new-checkout"
  }
}
```

#### POST /api/v1/flags/:key/evaluate

フラグを評価し、指定されたコンテキスト（ユーザー・テナント・属性）に基づいて有効/無効とバリアントを判定する。内部サービス用のエンドポイントであり、認証は不要。

**リクエスト**

```json
{
  "user_id": "user-001",
  "tenant_id": "tenant-abc",
  "attributes": {
    "environment": "production",
    "region": "ap-northeast-1"
  }
}
```

全フィールド省略可能。`attributes` 省略時は空マップ。

**レスポンス（200 OK -- フラグ有効）**

```json
{
  "flag_key": "enable-new-checkout",
  "enabled": true,
  "variant": "on",
  "reason": "flag is enabled"
}
```

**レスポンス（200 OK -- フラグ無効）**

```json
{
  "flag_key": "enable-new-checkout",
  "enabled": false,
  "variant": null,
  "reason": "flag is disabled"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "flag not found: enable-new-checkout"
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_FF_NOT_FOUND` | 404 | 指定されたフラグが見つからない |
| `SYS_FF_ALREADY_EXISTS` | 409 | 同一キーのフラグが既に存在する |
| `SYS_FF_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_FF_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

`api/proto/k1s0/system/featureflag/v1/featureflag.proto` に定義。

```protobuf
syntax = "proto3";
package k1s0.system.featureflag.v1;

import "k1s0/system/common/v1/types.proto";

service FeatureFlagService {
  rpc EvaluateFlag(EvaluateFlagRequest) returns (EvaluateFlagResponse);
  rpc GetFlag(GetFlagRequest) returns (GetFlagResponse);
  rpc CreateFlag(CreateFlagRequest) returns (CreateFlagResponse);
  rpc UpdateFlag(UpdateFlagRequest) returns (UpdateFlagResponse);
}

message EvaluateFlagRequest {
  string flag_key = 1;
  EvaluationContext context = 2;
}

message EvaluateFlagResponse {
  string flag_key = 1;
  bool enabled = 2;
  string variant = 3;
  string reason = 4;
}

message EvaluationContext {
  string user_id = 1;
  string tenant_id = 2;
  map<string, string> attributes = 3;
}

message GetFlagRequest {
  string flag_key = 1;
}

message GetFlagResponse {
  FeatureFlag flag = 1;
}

message CreateFlagRequest {
  string flag_key = 1;
  string description = 2;
  bool enabled = 3;
  repeated FlagVariant variants = 4;
}

message CreateFlagResponse {
  FeatureFlag flag = 1;
}

message UpdateFlagRequest {
  string flag_key = 1;
  bool enabled = 2;
  string description = 3;
}

message UpdateFlagResponse {
  FeatureFlag flag = 1;
}

message FeatureFlag {
  string id = 1;
  string flag_key = 2;
  string description = 3;
  bool enabled = 4;
  repeated FlagVariant variants = 5;
  k1s0.system.common.v1.Timestamp created_at = 6;
  k1s0.system.common.v1.Timestamp updated_at = 7;
}

message FlagVariant {
  string name = 1;
  string value = 2;
  int32 weight = 3;
}
```

---

## フラグ評価ロジック

### 評価フロー

フラグ評価は以下の順序で判定される。

```
1. フラグが存在するか確認（未存在: FlagNotFound エラー）
2. フラグが enabled=false の場合
   → EvaluationResult { enabled: false, variant: None, reason: "flag is disabled" }
3. フラグが enabled=true の場合
   → variants の先頭バリアントを選択
   → EvaluationResult { enabled: true, variant: Some(variants[0].name), reason: "flag is enabled" }
```

現在の実装では、ルール（FlagRule）による属性マッチング評価は未実装。ドメインモデルに `rules: Vec<FlagRule>` フィールドは定義済みであり、将来的にルールベースの条件分岐評価を追加予定。

### キャッシュ戦略

| 項目 | 値 |
| --- | --- |
| キャッシュライブラリ | moka v0.12 |
| キャッシュキー | `{flag_key}:{environment}:{user_id}` |
| TTL | 60 秒 |
| 最大エントリ数 | 10,000 |
| 無効化トリガー | フラグ更新・削除時に即座に無効化 + Kafka 通知受信時 |

---

## Kafka メッセージング設計

### フラグ変更通知

フラグの作成・更新・削除時に以下のメッセージを Kafka トピック `k1s0.system.featureflag.changed.v1` に送信する。

**メッセージフォーマット**

```json
{
  "event_type": "FLAG_UPDATED",
  "flag_key": "enable-new-checkout",
  "timestamp": "2026-02-20T12:30:00.000+00:00",
  "actor_user_id": "admin-001",
  "before": {
    "enabled": false,
    "variants": []
  },
  "after": {
    "enabled": true,
    "variants": [
      { "name": "on", "value": "true", "weight": 100 }
    ]
  }
}
```

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.featureflag.changed.v1` |
| acks | `all` |
| message.timeout.ms | `5000` |
| キー | フラグキー（例: `enable-new-checkout`） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `FeatureFlag`, `FlagEvaluation`, `FlagAuditLog` | エンティティ定義 |
| domain/repository | `FeatureFlagRepository`, `FlagAuditLogRepository` | リポジトリトレイト |
| domain/service | `FeatureFlagDomainService` | 評価ロジック・ロールアウト判定 |
| usecase | `EvaluateFlagUsecase`, `GetFlagUsecase`, `ListFlagsUsecase`, `CreateFlagUsecase`, `UpdateFlagUsecase`, `DeleteFlagUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `FeatureFlagPostgresRepository`, `FlagAuditLogPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/cache | `FlagCacheService` | moka キャッシュ実装 |
| infrastructure/messaging | `FlagChangeKafkaProducer` | Kafka プロデューサー（フラグ変更通知） |

### ドメインモデル

#### FeatureFlag

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | Uuid | フラグ ID（自動生成） |
| `flag_key` | String | フラグの一意キー（例: `enable-new-checkout`） |
| `description` | String | フラグの説明 |
| `enabled` | bool | フラグの有効/無効 |
| `variants` | Vec\<FlagVariant\> | バリアント定義リスト |
| `rules` | Vec\<FlagRule\> | ルール定義リスト |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### FlagVariant

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | String | バリアント名 |
| `value` | String | バリアント値 |
| `weight` | i32 | 重み（ロールアウト割合制御） |

#### FlagRule

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `attribute` | String | 評価対象属性名 |
| `operator` | String | 比較演算子（`eq`, `contains`, `in`） |
| `value` | String | 比較値 |
| `variant` | String | マッチ時に返すバリアント名 |

#### EvaluationContext

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `user_id` | Option\<String\> | 評価対象ユーザー ID |
| `tenant_id` | Option\<String\> | 評価対象テナント ID |
| `attributes` | HashMap\<String, String\> | 追加属性（ルール評価用） |

#### EvaluationResult

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `flag_key` | String | 評価対象のフラグキー |
| `enabled` | bool | 評価結果 |
| `variant` | Option\<String\> | 選択されたバリアント名 |
| `reason` | String | 評価理由 |

#### FlagAuditLog

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | 監査ログの一意識別子 |
| `flag_key` | String | 対象フラグキー |
| `action` | String | 操作種別（CREATE / UPDATE / DELETE） |
| `actor_user_id` | String | 操作者のユーザー ID |
| `before_value` | Option\<JSON\> | 変更前の値（CREATE 時は null） |
| `after_value` | Option\<JSON\> | 変更後の値（DELETE 時は null） |
| `trace_id` | String | OpenTelemetry トレース ID |
| `created_at` | DateTime\<Utc\> | 記録日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (flag_handler.rs)           │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_flags / get_flag / create_flag     │   │
                    │  │  update_flag / delete_flag               │   │
                    │  │  evaluate_flag                           │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (flag_grpc.rs)              │   │
                    │  │  EvaluateFlag / GetFlag / CreateFlag     │   │
                    │  │  UpdateFlag                              │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  EvaluateFlagUsecase / GetFlagUsecase /         │
                    │  ListFlagsUsecase / CreateFlagUsecase /         │
                    │  UpdateFlagUsecase / DeleteFlagUsecase          │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  FeatureFlag,   │              │ FeatureFlagRepository      │   │
    │  FlagEvaluation,│              │ FlagAuditLogRepository     │   │
    │  FlagAuditLog   │              │ (trait)                    │   │
    └────────────────┘              └──────────┬─────────────────┘   │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ FeatureFlagDomain           │                     │
                 │ Service        │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ FeatureFlagPostgres    │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ FlagAuditLogPostgres   │  │
                    │  │ moka Cache   │  │ Repository             │  │
                    │  │ Service      │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ Database               │  │
                    │  │ Config       │  │ Config                 │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "featureflag"
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
  topic: "k1s0.system.featureflag.changed.v1"

cache:
  max_entries: 10000
  ttl_seconds: 60
```

---

## デプロイ

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。featureflag 固有の values は以下の通り。

```yaml
# values-featureflag.yaml（infra/helm/services/system/featureflag/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/featureflag
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
    - path: "secret/data/k1s0/system/featureflag/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/featureflag/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-featureflag-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-featureflag-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-library-featureflag.md](../../libraries/config/featureflag.md) -- フィーチャーフラグクライアントライブラリ設計
