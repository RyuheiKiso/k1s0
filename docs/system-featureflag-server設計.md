# system-featureflag-server 設計

system tier のフィーチャーフラグサーバー設計を定義する。全サービスに動的な機能制御を提供し、フラグ定義を PostgreSQL で管理する。フラグ変更時は Kafka トピック `k1s0.system.featureflag.changed.v1` で全サービスに通知する。
Rust での実装を定義する。

## 概要

system tier のフィーチャーフラグサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| フラグ定義管理 | フィーチャーフラグの作成・更新・削除・一覧取得 |
| フラグ評価 | ユーザー・サービス・環境に基づくフラグの評価（有効/無効判定） |
| ロールアウト制御 | パーセンテージロールアウト・ユーザーセグメント・環境別制御 |
| 変更通知 | フラグ変更時に Kafka `k1s0.system.featureflag.changed.v1` で全サービスに通知 |
| 変更監査ログ | フラグ変更を PostgreSQL に記録し、変更前後の値を保存 |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC | tonic v0.12 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| キャッシュ | moka v0.12 |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/featureflag/` |

---

## 設計方針

[認証認可設計.md](認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| フラグ評価方式 | サーバー側評価。クライアントは gRPC/REST でフラグ値を問い合わせる |
| キャッシュ | moka で評価結果を TTL 60 秒キャッシュ。Kafka 通知受信時にキャッシュ無効化 |
| ロールアウト | パーセンテージ（0-100）・ユーザーリスト・環境名（development/staging/production）の AND 条件 |
| DB | PostgreSQL の `featureflag` スキーマ |
| Kafka | オプション。未設定時は変更通知なし（REST/gRPC API は動作する） |
| 監査ログ | フラグの作成・更新・削除時に変更前後の値を PostgreSQL に記録 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_FF_` とする。

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

フラグ一覧をページネーション付きで取得する。`environment` クエリパラメータで環境別にフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `environment` | string | No | - | 環境名でフィルタ（development/staging/production） |
| `enabled_only` | bool | No | false | 有効なフラグのみ取得 |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "flags": [
    {
      "key": "enable-new-checkout",
      "name": "新チェックアウトフロー",
      "description": "新しいチェックアウトフローを有効化する",
      "enabled": true,
      "rollout_percentage": 50,
      "target_environments": ["development", "staging"],
      "target_user_ids": ["user-001", "user-002"],
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 25,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### GET /api/v1/flags/:key

キー指定でフラグの詳細を取得する。

**レスポンス（200 OK）**

```json
{
  "key": "enable-new-checkout",
  "name": "新チェックアウトフロー",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": true,
  "rollout_percentage": 50,
  "target_environments": ["development", "staging"],
  "target_user_ids": ["user-001", "user-002"],
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
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

新しいフィーチャーフラグを作成する。フラグキーはシステム全体で一意でなければならない。作成時に監査ログを記録し、Kafka が設定されていれば変更通知を送信する。

**リクエスト**

```json
{
  "key": "enable-new-checkout",
  "name": "新チェックアウトフロー",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": false,
  "rollout_percentage": 0,
  "target_environments": ["development"],
  "target_user_ids": []
}
```

**レスポンス（201 Created）**

```json
{
  "key": "enable-new-checkout",
  "name": "新チェックアウトフロー",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": false,
  "rollout_percentage": 0,
  "target_environments": ["development"],
  "target_user_ids": [],
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_FF_ALREADY_EXISTS",
    "message": "feature flag already exists: enable-new-checkout",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_FF_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "key", "message": "key is required and must be non-empty"},
      {"field": "rollout_percentage", "message": "must be between 0 and 100"}
    ]
  }
}
```

#### PUT /api/v1/flags/:key

既存のフィーチャーフラグを更新する。更新時に変更前後の値を監査ログに記録し、Kafka が設定されていれば変更通知を送信する。キャッシュは即座に無効化される。

**リクエスト**

```json
{
  "name": "新チェックアウトフロー",
  "description": "新しいチェックアウトフローを有効化する（v2）",
  "enabled": true,
  "rollout_percentage": 50,
  "target_environments": ["development", "staging"],
  "target_user_ids": ["user-001", "user-002"]
}
```

**レスポンス（200 OK）**

```json
{
  "key": "enable-new-checkout",
  "name": "新チェックアウトフロー",
  "description": "新しいチェックアウトフローを有効化する（v2）",
  "enabled": true,
  "rollout_percentage": 50,
  "target_environments": ["development", "staging"],
  "target_user_ids": ["user-001", "user-002"],
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
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

#### DELETE /api/v1/flags/:key

フィーチャーフラグを削除する。削除時に監査ログを記録し、Kafka が設定されていれば変更通知を送信する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "feature flag enable-new-checkout deleted"
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

#### POST /api/v1/flags/:key/evaluate

フラグを評価し、指定されたコンテキスト（環境・ユーザー・サービス）に基づいて有効/無効を判定する。内部サービス用のエンドポイントであり、認証は不要。

**リクエスト**

```json
{
  "environment": "production",
  "user_id": "user-001",
  "service_name": "order-service"
}
```

**レスポンス（200 OK -- フラグ有効）**

```json
{
  "enabled": true,
  "reason": "user_id in target_user_ids"
}
```

**レスポンス（200 OK -- フラグ無効）**

```json
{
  "enabled": false,
  "reason": "environment not in target_environments"
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

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_FF_NOT_FOUND` | 404 | 指定されたフラグが見つからない |
| `SYS_FF_ALREADY_EXISTS` | 409 | 同一キーのフラグが既に存在する |
| `SYS_FF_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_FF_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.featureflag.v1;

service FeatureFlagService {
  rpc EvaluateFlag(EvaluateFlagRequest) returns (EvaluateFlagResponse);
  rpc GetFlag(GetFlagRequest) returns (GetFlagResponse);
  rpc ListFlags(ListFlagsRequest) returns (ListFlagsResponse);
  rpc UpdateFlag(UpdateFlagRequest) returns (UpdateFlagResponse);
}

message EvaluateFlagRequest {
  string key = 1;
  string environment = 2;
  optional string user_id = 3;
  optional string service_name = 4;
}

message EvaluateFlagResponse {
  bool enabled = 1;
  string reason = 2;
}

message GetFlagRequest {
  string key = 1;
}

message GetFlagResponse {
  FeatureFlag flag = 1;
}

message ListFlagsRequest {
  string environment = 1;
  bool enabled_only = 2;
}

message ListFlagsResponse {
  repeated FeatureFlag flags = 1;
}

message UpdateFlagRequest {
  string key = 1;
  bool enabled = 2;
  optional uint32 rollout_percentage = 3;
  repeated string target_environments = 4;
}

message UpdateFlagResponse {
  FeatureFlag flag = 1;
}

message FeatureFlag {
  string key = 1;
  string name = 2;
  string description = 3;
  bool enabled = 4;
  uint32 rollout_percentage = 5;
  repeated string target_environments = 6;
  repeated string target_user_ids = 7;
  string created_at = 8;
  string updated_at = 9;
}
```

---

## フラグ評価ロジック

### 評価フロー

フラグ評価は以下の順序で判定される。すべての条件を AND で評価する。

```
1. フラグが存在するか確認（未存在: NOT_FOUND）
2. フラグが enabled=false の場合 → 無効（reason: "flag is disabled"）
3. 環境チェック: target_environments が空でなく、指定環境が含まれない場合 → 無効
4. ユーザーチェック: target_user_ids が空でなく、指定ユーザーが含まれる場合 → 有効（ホワイトリスト）
5. ロールアウト判定: rollout_percentage に基づくハッシュ判定
   - hash(flag_key + user_id) % 100 < rollout_percentage → 有効
   - user_id 未指定時は rollout_percentage > 0 なら有効
```

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
    "rollout_percentage": 0
  },
  "after": {
    "enabled": true,
    "rollout_percentage": 50
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

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース・ドメインサービス）
  ^
usecase（ビジネスロジック）
  ^
adapter（REST ハンドラー・gRPC ハンドラー）
  ^
infrastructure（DB接続・Kafka Producer・moka キャッシュ・設定ローダー）
```

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
                    │  │  EvaluateFlag / GetFlag / ListFlags      │   │
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

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。featureflag 固有の values は以下の通り。

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

- [system-featureflag-server-実装設計.md](system-featureflag-server-実装設計.md) -- 実装設計の詳細
- [system-featureflag-server-デプロイ設計.md](system-featureflag-server-デプロイ設計.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

- [system-library-featureflag設計.md](system-library-featureflag設計.md) -- フィーチャーフラグクライアントライブラリ設計
- [system-library-概要.md](system-library-概要.md) -- ライブラリ一覧
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [認証認可設計.md](認証認可設計.md) -- RBAC 認可モデル
- [メッセージング設計.md](メッセージング設計.md) -- Kafka メッセージング設計
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [config設計.md](config設計.md) -- config.yaml スキーマ
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
