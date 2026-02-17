# system-config-server 設計

system tier の設定管理サーバー設計を定義する。全サービスに対して REST と gRPC で設定値を提供し、設定変更時の通知・監査ログ記録を行う。
Go と Rust の両実装を対等に定義する。

## 概要

system tier の設定管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| 設定値の取得 API | namespace.key 形式のキーベースで設定値を取得する |
| 設定値の一括取得 | サービス名を指定して必要な設定を一括取得する |
| 設定値の更新 API | 設定値の動的更新（sys_admin / sys_operator 権限が必要） |
| 設定変更の監査ログ | 全ての設定変更を監査ログとして PostgreSQL に記録し、Kafka に非同期配信する |
| 設定変更通知 | Kafka トピック `k1s0.system.config.changed.v1` で設定変更を依存サービスに通知する |

### 技術スタック

| コンポーネント | Go | Rust |
| --- | --- | --- |
| HTTP フレームワーク | gin v1.10 | axum + tokio |
| gRPC フレームワーク | google.golang.org/grpc v1.68 | tonic v0.12 |
| DB アクセス | github.com/jmoiron/sqlx | sqlx v0.8 |
| Kafka | github.com/segmentio/kafka-go | rdkafka (rust-rdkafka) |
| OTel | go.opentelemetry.io/otel v1.31 | opentelemetry v0.27 |
| 設定管理 | gopkg.in/yaml.v3 | serde_yaml |
| バリデーション | go-playground/validator/v10 | validator v0.18 |
| キャッシュ | github.com/dgraph-io/ristretto | moka v0.12 |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Go | `regions/system/server/go/config/` |
| Rust | `regions/system/server/rust/config/` |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_CONFIG_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/config/:namespace/:key` | 設定値取得 | `sys_auditor` 以上 |
| GET | `/api/v1/config/:namespace` | namespace 内の設定値一覧 | `sys_auditor` 以上 |
| PUT | `/api/v1/config/:namespace/:key` | 設定値更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/config/:namespace/:key` | 設定値削除 | `sys_admin` |
| GET | `/api/v1/config/services/:service_name` | サービス向け設定一括取得 | Bearer token required |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

#### GET /api/v1/config/:namespace/:key

指定された namespace とキーに対応する設定値を取得する。

**レスポンス（200 OK）**

```json
{
  "namespace": "system.auth.database",
  "key": "max_connections",
  "value": 25,
  "version": 3,
  "description": "認証サーバーの DB 最大接続数",
  "updated_by": "admin@example.com",
  "updated_at": "2026-02-15T14:30:00Z"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_CONFIG_KEY_NOT_FOUND",
    "message": "指定された設定キーが見つかりません",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/config/:namespace

namespace 内の全設定値をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1ページあたりの件数（最大 100） |
| `search` | string | No | - | キー名で部分一致検索 |

**レスポンス（200 OK）**

```json
{
  "entries": [
    {
      "namespace": "system.auth.database",
      "key": "max_connections",
      "value": 25,
      "version": 3,
      "description": "認証サーバーの DB 最大接続数",
      "updated_by": "admin@example.com",
      "updated_at": "2026-02-15T14:30:00Z"
    },
    {
      "namespace": "system.auth.database",
      "key": "ssl_mode",
      "value": "require",
      "version": 1,
      "description": "SSL 接続モード",
      "updated_by": "admin@example.com",
      "updated_at": "2026-01-10T09:00:00Z"
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

#### PUT /api/v1/config/:namespace/:key

設定値を更新する。楽観的排他制御のため、リクエストに現在の `version` を含める必要がある。

**リクエスト**

```json
{
  "value": 50,
  "version": 3,
  "description": "認証サーバーの DB 最大接続数（増設）"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `value` | any (JSON) | Yes | 設定値（string, number, boolean, object） |
| `version` | int | Yes | 現在のバージョン番号（楽観的排他制御） |
| `description` | string | No | 設定の説明文 |

**レスポンス（200 OK）**

```json
{
  "namespace": "system.auth.database",
  "key": "max_connections",
  "value": 50,
  "version": 4,
  "description": "認証サーバーの DB 最大接続数（増設）",
  "updated_by": "operator@example.com",
  "updated_at": "2026-02-17T10:30:00Z"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_CONFIG_VERSION_CONFLICT",
    "message": "設定値が他のユーザーによって更新されています。最新のバージョンを取得してください",
    "request_id": "req_abc123def456",
    "details": [
      {
        "field": "version",
        "message": "期待値: 3, 現在値: 4"
      }
    ]
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_CONFIG_VALIDATION_FAILED",
    "message": "リクエストのバリデーションに失敗しました",
    "request_id": "req_abc123def456",
    "details": [
      {
        "field": "value",
        "message": "value フィールドは必須です"
      }
    ]
  }
}
```

#### DELETE /api/v1/config/:namespace/:key

設定値を削除する。`sys_admin` ロールのみ実行可能。

**レスポンス（204 No Content）**

ボディなし。

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_CONFIG_KEY_NOT_FOUND",
    "message": "指定された設定キーが見つかりません",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/config/services/:service_name

サービス名を指定して、そのサービスに必要な設定値を一括取得する。サービス間通信で利用する。Bearer トークンによる認証が必要。

**レスポンス（200 OK）**

```json
{
  "service_name": "auth-server",
  "entries": [
    {
      "namespace": "system.auth.database",
      "key": "max_connections",
      "value": 25
    },
    {
      "namespace": "system.auth.database",
      "key": "ssl_mode",
      "value": "require"
    },
    {
      "namespace": "system.auth.jwt",
      "key": "issuer",
      "value": "https://auth.k1s0.internal.example.com/realms/k1s0"
    }
  ]
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_CONFIG_SERVICE_NOT_FOUND",
    "message": "指定されたサービスの設定が見つかりません",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /healthz

**レスポンス（200 OK）**

```json
{
  "status": "ok"
}
```

#### GET /readyz

PostgreSQL と Kafka への接続を確認する。

**レスポンス（200 OK）**

```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "kafka": "ok"
  }
}
```

**レスポンス（503 Service Unavailable）**

```json
{
  "status": "not ready",
  "checks": {
    "database": "ok",
    "kafka": "error: connection timeout"
  }
}
```

### gRPC サービス定義

proto ファイルは [API設計.md](API設計.md) D-009 の命名規則に従い、サービス内の `api/proto/` に配置する。

```
{config-server}/api/proto/
└── k1s0/
    └── system/
        └── config/
            └── v1/
                └── config.proto
```

```protobuf
// k1s0/system/config/v1/config.proto
syntax = "proto3";
package k1s0.system.config.v1;

import "k1s0/system/common/v1/types.proto";

service ConfigService {
  // 設定値取得
  rpc GetConfig(GetConfigRequest) returns (GetConfigResponse);

  // namespace 内の設定値一覧取得
  rpc ListConfigs(ListConfigsRequest) returns (ListConfigsResponse);

  // サービス向け設定一括取得
  rpc GetServiceConfig(GetServiceConfigRequest) returns (GetServiceConfigResponse);

  // 設定変更の監視（Server-Side Streaming）
  rpc WatchConfig(WatchConfigRequest) returns (stream ConfigChangeEvent);
}

// --- Get Config ---

message GetConfigRequest {
  string namespace = 1;
  string key = 2;
}

message GetConfigResponse {
  ConfigEntry entry = 1;
}

message ConfigEntry {
  string id = 1;
  string namespace = 2;
  string key = 3;
  bytes value = 4;             // JSON エンコード済みの値
  int32 version = 5;
  string description = 6;
  string created_by = 7;
  string updated_by = 8;
  k1s0.system.common.v1.Timestamp created_at = 9;
  k1s0.system.common.v1.Timestamp updated_at = 10;
}

// --- List Configs ---

message ListConfigsRequest {
  string namespace = 1;
  k1s0.system.common.v1.Pagination pagination = 2;
  string search = 3;           // キー名の部分一致検索
}

message ListConfigsResponse {
  repeated ConfigEntry entries = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

// --- Service Config ---

message GetServiceConfigRequest {
  string service_name = 1;
  string environment = 2;  // dev | staging | prod
}

message GetServiceConfigResponse {
  map<string, string> configs = 1;  // flattened key-value pairs
}

// --- Watch Config ---

message WatchConfigRequest {
  repeated string namespaces = 1;  // 監視対象の namespace 一覧（空の場合は全件）
}

message ConfigChangeEvent {
  string namespace = 1;
  string key = 2;
  bytes old_value = 3;         // 変更前の値（JSON エンコード済み）
  bytes new_value = 4;         // 変更後の値（JSON エンコード済み）
  int32 old_version = 5;
  int32 new_version = 6;
  string changed_by = 7;
  string change_type = 8;      // "CREATED", "UPDATED", "DELETED"
  k1s0.system.common.v1.Timestamp changed_at = 9;
}
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の4レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース）
  ^
usecase（ビジネスロジック）
  ^
adapter（ハンドラー・プレゼンター・ゲートウェイ）
  ^
infra（DB接続・Kafka プロデューサー・キャッシュ・設定ローダー）
```

| レイヤー | パッケージ / モジュール | 責務 |
| --- | --- | --- |
| domain/model | `ConfigEntry`, `ConfigChangeLog` | エンティティ定義 |
| domain/repository | `ConfigRepository`, `ConfigChangeLogRepository` | リポジトリインターフェース / トレイト |
| domain/service | `ConfigDomainService` | ドメインサービス（namespace バリデーション・バージョン検証ロジック） |
| usecase | `GetConfigUsecase`, `ListConfigsUsecase`, `UpdateConfigUsecase`, `DeleteConfigUsecase`, `GetServiceConfigUsecase` | ユースケース |
| adapter/handler | REST ハンドラー, gRPC ハンドラー | プロトコル変換 |
| adapter/presenter | レスポンスフォーマット | ドメインモデル → API レスポンス変換 |
| adapter/gateway | （外部サービスなし） | - |
| infra/persistence | PostgreSQL リポジトリ実装 | 設定値・変更ログの永続化 |
| infra/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infra/messaging | Kafka プロデューサー | 設定変更イベントの非同期配信 |
| infra/cache | インメモリキャッシュ | 設定値のキャッシュ管理（TTL 制御） |

### ドメインモデル

#### ConfigEntry

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string (UUID) | 設定エントリの一意識別子 |
| `namespace` | string | 名前空間（`{tier}.{service}.{section}` 形式、例: `system.auth.database`） |
| `key` | string | 設定キー名（例: `max_connections`） |
| `value` | JSON | 設定値（JSONB。string, number, boolean, object を格納可能） |
| `version` | int | バージョン番号（楽観的排他制御用。更新のたびにインクリメント） |
| `description` | string | 設定の説明文 |
| `created_by` | string | 作成者 |
| `updated_by` | string | 最終更新者 |
| `created_at` | timestamp | 作成日時 |
| `updated_at` | timestamp | 最終更新日時 |

#### ConfigChangeLog

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string (UUID) | 変更ログの一意識別子 |
| `config_entry_id` | string (UUID) | 対象の設定エントリ ID |
| `namespace` | string | 名前空間 |
| `key` | string | 設定キー名 |
| `old_value` | JSON | 変更前の値（新規作成時は null） |
| `new_value` | JSON | 変更後の値（削除時は null） |
| `old_version` | int | 変更前のバージョン |
| `new_version` | int | 変更後のバージョン |
| `change_type` | string | 変更種別（`CREATED`, `UPDATED`, `DELETED`） |
| `changed_by` | string | 変更者 |
| `changed_at` | timestamp | 変更日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────────┐
                    │                    adapter 層                       │
                    │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
                    │  │ REST Handler │  │ gRPC Handler │  │ Presenter│ │
                    │  └──────┬───────┘  └──────┬───────┘  └─────┬────┘ │
                    │         │                  │                │      │
                    └─────────┼──────────────────┼────────────────┼──────┘
                              │                  │                │
                    ┌─────────▼──────────────────▼────────────────▼──────┐
                    │                   usecase 層                       │
                    │  GetConfig / ListConfigs / UpdateConfig /          │
                    │  DeleteConfig / GetServiceConfig                   │
                    └─────────┬──────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────────────┐
              │               │                       │
    ┌─────────▼──────┐  ┌────▼───────────┐  ┌───────▼─────────────┐
    │  domain/model   │  │ domain/service │  │ domain/repository   │
    │  ConfigEntry,   │  │ ConfigDomain   │  │ ConfigRepository    │
    │  ConfigChange   │  │ Service        │  │ ConfigChangeLog     │
    │  Log            │  │                │  │ Repository          │
    └────────────────┘  └────────────────┘  │ (interface/trait)    │
                                            └──────────┬──────────┘
                                                       │
                    ┌──────────────────────────────────┼──────────────┐
                    │                  infra 層         │              │
                    │  ┌──────────────┐  ┌─────────────▼──────────┐  │
                    │  │ In-Memory    │  │ PostgreSQL Repository  │  │
                    │  │ Cache        │  │ (impl)                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ Config       │  │ Kafka Producer         │  │
                    │  │ Loader       │  │ (change events)        │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## Go 実装 (regions/system/server/go/config/)

### ディレクトリ構成

```
regions/system/server/go/config/
├── cmd/
│   └── main.go                          # エントリポイント
├── internal/
│   ├── domain/
│   │   ├── model/
│   │   │   ├── config_entry.go          # ConfigEntry エンティティ
│   │   │   └── config_change_log.go     # ConfigChangeLog エンティティ
│   │   ├── repository/
│   │   │   ├── config_repository.go     # ConfigRepository インターフェース
│   │   │   ├── config_change_log_repository.go  # ConfigChangeLogRepository インターフェース
│   │   │   └── mock_*.go               # gomock 生成モック
│   │   └── service/
│   │       └── config_domain_service.go # namespace バリデーション・バージョン検証
│   ├── usecase/
│   │   ├── get_config.go                # 設定値取得
│   │   ├── list_configs.go              # 設定値一覧
│   │   ├── update_config.go             # 設定値更新
│   │   ├── delete_config.go             # 設定値削除
│   │   ├── get_service_config.go        # サービス向け設定一括取得
│   │   └── *_test.go                    # 各ユースケースのテスト
│   ├── adapter/
│   │   ├── handler/
│   │   │   ├── rest_handler.go          # REST ハンドラー
│   │   │   ├── grpc_handler.go          # gRPC ハンドラー
│   │   │   ├── error.go                 # エラーレスポンス
│   │   │   └── *_test.go               # ハンドラーテスト
│   │   ├── presenter/
│   │   │   └── response.go              # レスポンスフォーマット
│   │   └── middleware/
│   │       ├── auth.go                  # JWT 認証ミドルウェア
│   │       └── rbac.go                  # RBAC ミドルウェア
│   └── infra/
│       ├── config/
│       │   ├── config.go                # Config 構造体・ローダー
│       │   └── logger.go               # 構造化ログ初期化
│       ├── persistence/
│       │   ├── db.go                    # DB 接続
│       │   ├── config_repository.go     # ConfigRepository 実装
│       │   ├── config_change_log_repository.go  # ConfigChangeLogRepository 実装
│       │   ├── migrations/
│       │   │   ├── 001_create_config_entries.sql
│       │   │   └── 002_create_config_change_logs.sql
│       │   └── *_test.go               # リポジトリテスト
│       ├── cache/
│       │   ├── config_cache.go          # インメモリキャッシュ
│       │   └── config_cache_test.go
│       └── messaging/
│           ├── producer.go              # Kafka プロデューサー
│           └── producer_test.go
├── api/
│   ├── openapi/
│   │   ├── openapi.yaml                 # OpenAPI 定義
│   │   └── gen.yaml                     # oapi-codegen 設定
│   └── proto/
│       └── k1s0/system/config/v1/
│           └── config.proto             # gRPC サービス定義
├── config/
│   ├── config.yaml                      # デフォルト設定
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── go.mod
├── go.sum
├── Dockerfile
└── README.md
```

### go.mod

```go
module github.com/k1s0/regions/system/server/go/config

go 1.23

require (
    github.com/dgraph-io/ristretto v0.1.1
    github.com/gin-gonic/gin v1.10.0
    github.com/go-playground/validator/v10 v10.22.1
    github.com/jmoiron/sqlx v1.4.0
    github.com/lib/pq v1.10.9
    github.com/oapi-codegen/oapi-codegen/v2 v2.4.1
    github.com/oapi-codegen/runtime v1.1.1
    github.com/segmentio/kafka-go v0.4.47
    github.com/stretchr/testify v1.9.0
    go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin v0.56.0
    go.opentelemetry.io/otel v1.31.0
    go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc v1.31.0
    go.opentelemetry.io/otel/sdk v1.31.0
    go.uber.org/mock v0.5.0
    google.golang.org/grpc v1.68.0
    google.golang.org/protobuf v1.35.2
    gopkg.in/yaml.v3 v3.0.1
)
```

### cmd/main.go

```go
package main

import (
    "context"
    "fmt"
    "log/slog"
    "net"
    "net/http"
    "os"
    "os/signal"
    "syscall"
    "time"

    "github.com/gin-gonic/gin"
    "github.com/prometheus/client_golang/prometheus/promhttp"
    "go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin"
    "google.golang.org/grpc"

    "github.com/k1s0/regions/system/server/go/config/internal/adapter/handler"
    "github.com/k1s0/regions/system/server/go/config/internal/adapter/middleware"
    "github.com/k1s0/regions/system/server/go/config/internal/domain/service"
    "github.com/k1s0/regions/system/server/go/config/internal/infra/cache"
    infraconfig "github.com/k1s0/regions/system/server/go/config/internal/infra/config"
    "github.com/k1s0/regions/system/server/go/config/internal/infra/messaging"
    "github.com/k1s0/regions/system/server/go/config/internal/infra/persistence"
    "github.com/k1s0/regions/system/server/go/config/internal/usecase"
)

func main() {
    // --- Config ---
    cfg, err := infraconfig.Load("config/config.yaml")
    if err != nil {
        slog.Error("failed to load config", "error", err)
        os.Exit(1)
    }
    if err := cfg.Validate(); err != nil {
        slog.Error("config validation failed", "error", err)
        os.Exit(1)
    }

    // --- Logger ---
    logger := infraconfig.NewLogger(
        cfg.App.Environment, cfg.App.Name, cfg.App.Version, cfg.App.Tier,
    )
    slog.SetDefault(logger)

    // --- OpenTelemetry ---
    tp, err := infraconfig.InitTracer(context.Background(), cfg.App.Name)
    if err != nil {
        slog.Error("failed to init tracer", "error", err)
        os.Exit(1)
    }
    defer tp.Shutdown(context.Background())

    // --- Database ---
    db, err := persistence.NewDB(cfg.Database)
    if err != nil {
        slog.Error("failed to connect database", "error", err)
        os.Exit(1)
    }
    defer db.Close()

    // --- Kafka ---
    producer := messaging.NewProducer(cfg.Kafka)
    defer producer.Close()

    // --- Cache ---
    configCache := cache.NewConfigCache(cfg.ConfigServer.Cache.TTL, cfg.ConfigServer.Cache.MaxEntries)

    // --- DI ---
    configRepo := persistence.NewConfigRepository(db)
    changeLogRepo := persistence.NewConfigChangeLogRepository(db)
    configDomainSvc := service.NewConfigDomainService()

    // Usecases
    getConfigUC := usecase.NewGetConfigUseCase(configRepo, configCache)
    listConfigsUC := usecase.NewListConfigsUseCase(configRepo)
    updateConfigUC := usecase.NewUpdateConfigUseCase(
        configRepo, changeLogRepo, configDomainSvc, configCache, producer,
    )
    deleteConfigUC := usecase.NewDeleteConfigUseCase(
        configRepo, changeLogRepo, configCache, producer,
    )
    getServiceConfigUC := usecase.NewGetServiceConfigUseCase(configRepo, configCache)

    // --- REST Router ---
    r := gin.New()
    r.Use(gin.Recovery())
    r.Use(otelgin.Middleware(cfg.App.Name))
    r.Use(middleware.RequestID())

    // ヘルスチェック
    r.GET("/healthz", func(c *gin.Context) {
        c.JSON(http.StatusOK, gin.H{"status": "ok"})
    })
    r.GET("/readyz", handler.ReadyzHandler(db, producer))
    r.GET("/metrics", gin.WrapH(promhttp.Handler()))

    // API ルート
    restHandler := handler.NewRESTHandler(
        getConfigUC, listConfigsUC, updateConfigUC,
        deleteConfigUC, getServiceConfigUC,
    )
    restHandler.RegisterRoutes(r)

    // --- gRPC Server ---
    grpcServer := grpc.NewServer(
        grpc.ChainUnaryInterceptor(
            handler.UnaryLoggingInterceptor(logger),
            handler.UnaryTracingInterceptor(),
        ),
    )
    grpcHandler := handler.NewGRPCHandler(
        getConfigUC, listConfigsUC, getServiceConfigUC,
    )
    grpcHandler.Register(grpcServer)

    // --- Start Servers ---
    // REST
    srv := &http.Server{
        Addr:         fmt.Sprintf(":%d", cfg.Server.Port),
        Handler:      r,
        ReadTimeout:  cfg.Server.ReadTimeout,
        WriteTimeout: cfg.Server.WriteTimeout,
    }
    go func() {
        slog.Info("REST server starting", "port", cfg.Server.Port)
        if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
            slog.Error("REST server failed", "error", err)
            os.Exit(1)
        }
    }()

    // gRPC
    go func() {
        lis, err := net.Listen("tcp", fmt.Sprintf(":%d", cfg.GRPC.Port))
        if err != nil {
            slog.Error("gRPC listen failed", "error", err)
            os.Exit(1)
        }
        slog.Info("gRPC server starting", "port", cfg.GRPC.Port)
        if err := grpcServer.Serve(lis); err != nil {
            slog.Error("gRPC server failed", "error", err)
            os.Exit(1)
        }
    }()

    // --- Graceful Shutdown ---
    quit := make(chan os.Signal, 1)
    signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
    <-quit
    slog.Info("shutting down servers...")

    ctx, cancel := context.WithTimeout(context.Background(), cfg.Server.ShutdownTimeout)
    defer cancel()

    grpcServer.GracefulStop()
    if err := srv.Shutdown(ctx); err != nil {
        slog.Error("REST server forced to shutdown", "error", err)
    }
    slog.Info("servers exited")
}
```

### ドメインモデル（Go）

```go
// internal/domain/model/config_entry.go
package model

import (
    "encoding/json"
    "time"
)

// ConfigEntry は設定エントリを表すドメインエンティティ。
type ConfigEntry struct {
    ID          string          `json:"id" db:"id"`
    Namespace   string          `json:"namespace" db:"namespace"`
    Key         string          `json:"key" db:"key"`
    Value       json.RawMessage `json:"value" db:"value"`
    Version     int             `json:"version" db:"version"`
    Description string          `json:"description" db:"description"`
    CreatedBy   string          `json:"created_by" db:"created_by"`
    UpdatedBy   string          `json:"updated_by" db:"updated_by"`
    CreatedAt   time.Time       `json:"created_at" db:"created_at"`
    UpdatedAt   time.Time       `json:"updated_at" db:"updated_at"`
}
```

```go
// internal/domain/model/config_change_log.go
package model

import (
    "encoding/json"
    "time"
)

// ConfigChangeLog は設定変更の監査ログを表す。
type ConfigChangeLog struct {
    ID            string          `json:"id" db:"id"`
    ConfigEntryID string          `json:"config_entry_id" db:"config_entry_id"`
    Namespace     string          `json:"namespace" db:"namespace"`
    Key           string          `json:"key" db:"key"`
    OldValue      json.RawMessage `json:"old_value,omitempty" db:"old_value"`
    NewValue      json.RawMessage `json:"new_value,omitempty" db:"new_value"`
    OldVersion    int             `json:"old_version" db:"old_version"`
    NewVersion    int             `json:"new_version" db:"new_version"`
    ChangeType    string          `json:"change_type" db:"change_type"` // CREATED, UPDATED, DELETED
    ChangedBy     string          `json:"changed_by" db:"changed_by"`
    ChangedAt     time.Time       `json:"changed_at" db:"changed_at"`
}
```

### リポジトリインターフェース（Go）

```go
// internal/domain/repository/config_repository.go
package repository

//go:generate mockgen -source=config_repository.go -destination=mock_config_repository.go -package=repository

import (
    "context"

    "github.com/k1s0/regions/system/server/go/config/internal/domain/model"
)

// ConfigRepository は設定エントリの永続化インターフェース。
type ConfigRepository interface {
    FindByNamespaceAndKey(ctx context.Context, namespace, key string) (*model.ConfigEntry, error)
    ListByNamespace(ctx context.Context, namespace string, params ListParams) ([]*model.ConfigEntry, int, error)
    ListByServiceName(ctx context.Context, serviceName string) ([]*model.ConfigEntry, error)
    Create(ctx context.Context, entry *model.ConfigEntry) error
    Update(ctx context.Context, entry *model.ConfigEntry) error
    Delete(ctx context.Context, namespace, key string) error
}

// ListParams はリスト取得パラメータ。
type ListParams struct {
    Search   string
    Page     int
    PageSize int
}
```

```go
// internal/domain/repository/config_change_log_repository.go
package repository

//go:generate mockgen -source=config_change_log_repository.go -destination=mock_config_change_log_repository.go -package=repository

import (
    "context"

    "github.com/k1s0/regions/system/server/go/config/internal/domain/model"
)

// ConfigChangeLogRepository は設定変更ログの永続化インターフェース。
type ConfigChangeLogRepository interface {
    Create(ctx context.Context, log *model.ConfigChangeLog) error
    ListByConfigEntryID(ctx context.Context, configEntryID string, page, pageSize int) ([]*model.ConfigChangeLog, int, error)
}
```

### ユースケース（Go）

```go
// internal/usecase/get_config.go
package usecase

import (
    "context"

    "go.opentelemetry.io/otel"

    "github.com/k1s0/regions/system/server/go/config/internal/domain/model"
    "github.com/k1s0/regions/system/server/go/config/internal/domain/repository"
    "github.com/k1s0/regions/system/server/go/config/internal/infra/cache"
)

// GetConfigUseCase は設定値取得ユースケース。
type GetConfigUseCase struct {
    repo  repository.ConfigRepository
    cache *cache.ConfigCache
}

func NewGetConfigUseCase(repo repository.ConfigRepository, cache *cache.ConfigCache) *GetConfigUseCase {
    return &GetConfigUseCase{
        repo:  repo,
        cache: cache,
    }
}

// Execute は namespace と key から設定値を取得する。キャッシュを優先し、ミス時に DB から取得する。
func (uc *GetConfigUseCase) Execute(ctx context.Context, namespace, key string) (*model.ConfigEntry, error) {
    ctx, span := otel.Tracer("config-server").Start(ctx, "GetConfigUseCase.Execute")
    defer span.End()

    // キャッシュから取得を試みる
    cacheKey := namespace + ":" + key
    if entry, ok := uc.cache.Get(cacheKey); ok {
        return entry, nil
    }

    // DB から取得
    entry, err := uc.repo.FindByNamespaceAndKey(ctx, namespace, key)
    if err != nil {
        return nil, err
    }

    // キャッシュに格納
    uc.cache.Set(cacheKey, entry)

    return entry, nil
}
```

```go
// internal/usecase/update_config.go
package usecase

import (
    "context"
    "encoding/json"
    "time"

    "go.opentelemetry.io/otel"

    "github.com/k1s0/regions/system/server/go/config/internal/domain/model"
    "github.com/k1s0/regions/system/server/go/config/internal/domain/repository"
    "github.com/k1s0/regions/system/server/go/config/internal/domain/service"
    "github.com/k1s0/regions/system/server/go/config/internal/infra/cache"
    "github.com/k1s0/regions/system/server/go/config/internal/infra/messaging"
)

// UpdateConfigUseCase は設定値更新ユースケース。
type UpdateConfigUseCase struct {
    repo          repository.ConfigRepository
    changeLogRepo repository.ConfigChangeLogRepository
    domainSvc     *service.ConfigDomainService
    cache         *cache.ConfigCache
    producer      *messaging.Producer
}

func NewUpdateConfigUseCase(
    repo repository.ConfigRepository,
    changeLogRepo repository.ConfigChangeLogRepository,
    domainSvc *service.ConfigDomainService,
    cache *cache.ConfigCache,
    producer *messaging.Producer,
) *UpdateConfigUseCase {
    return &UpdateConfigUseCase{
        repo:          repo,
        changeLogRepo: changeLogRepo,
        domainSvc:     domainSvc,
        cache:         cache,
        producer:      producer,
    }
}

// UpdateConfigInput は設定値更新の入力パラメータ。
type UpdateConfigInput struct {
    Namespace   string
    Key         string
    Value       json.RawMessage
    Version     int    // 楽観的排他制御用
    Description string
    UpdatedBy   string
}

// Execute は設定値を更新し、変更ログ記録・キャッシュ無効化・Kafka 通知を行う。
func (uc *UpdateConfigUseCase) Execute(ctx context.Context, input UpdateConfigInput) (*model.ConfigEntry, error) {
    ctx, span := otel.Tracer("config-server").Start(ctx, "UpdateConfigUseCase.Execute")
    defer span.End()

    // namespace バリデーション
    if err := uc.domainSvc.ValidateNamespace(input.Namespace); err != nil {
        return nil, err
    }

    // 現在の設定値を取得
    current, err := uc.repo.FindByNamespaceAndKey(ctx, input.Namespace, input.Key)
    if err != nil {
        return nil, err
    }

    // バージョン検証（楽観的排他制御）
    if err := uc.domainSvc.ValidateVersion(current.Version, input.Version); err != nil {
        return nil, ErrVersionConflict
    }

    // 更新
    oldValue := current.Value
    current.Value = input.Value
    current.Version++
    current.Description = input.Description
    current.UpdatedBy = input.UpdatedBy
    current.UpdatedAt = time.Now()

    if err := uc.repo.Update(ctx, current); err != nil {
        return nil, err
    }

    // 変更ログ記録
    changeLog := &model.ConfigChangeLog{
        ConfigEntryID: current.ID,
        Namespace:     current.Namespace,
        Key:           current.Key,
        OldValue:      oldValue,
        NewValue:      current.Value,
        OldVersion:    input.Version,
        NewVersion:    current.Version,
        ChangeType:    "UPDATED",
        ChangedBy:     input.UpdatedBy,
        ChangedAt:     current.UpdatedAt,
    }
    _ = uc.changeLogRepo.Create(ctx, changeLog)

    // キャッシュ無効化
    cacheKey := input.Namespace + ":" + input.Key
    uc.cache.Delete(cacheKey)

    // Kafka 通知
    uc.producer.PublishConfigChanged(ctx, changeLog)

    return current, nil
}
```

### REST ハンドラー（Go）

```go
// internal/adapter/handler/rest_handler.go
package handler

import (
    "net/http"

    "github.com/gin-gonic/gin"

    "github.com/k1s0/regions/system/server/go/config/internal/adapter/middleware"
    "github.com/k1s0/regions/system/server/go/config/internal/usecase"
)

type RESTHandler struct {
    getConfigUC        *usecase.GetConfigUseCase
    listConfigsUC      *usecase.ListConfigsUseCase
    updateConfigUC     *usecase.UpdateConfigUseCase
    deleteConfigUC     *usecase.DeleteConfigUseCase
    getServiceConfigUC *usecase.GetServiceConfigUseCase
}

func NewRESTHandler(
    getConfigUC *usecase.GetConfigUseCase,
    listConfigsUC *usecase.ListConfigsUseCase,
    updateConfigUC *usecase.UpdateConfigUseCase,
    deleteConfigUC *usecase.DeleteConfigUseCase,
    getServiceConfigUC *usecase.GetServiceConfigUseCase,
) *RESTHandler {
    return &RESTHandler{
        getConfigUC:        getConfigUC,
        listConfigsUC:      listConfigsUC,
        updateConfigUC:     updateConfigUC,
        deleteConfigUC:     deleteConfigUC,
        getServiceConfigUC: getServiceConfigUC,
    }
}

func (h *RESTHandler) RegisterRoutes(r *gin.Engine) {
    v1 := r.Group("/api/v1")

    // 設定値管理（sys_auditor 以上が読み取り可能）
    config := v1.Group("/config")
    {
        config.GET("/:namespace/:key",
            middleware.RequirePermission("read", "config"),
            h.GetConfig,
        )
        config.GET("/:namespace",
            middleware.RequirePermission("read", "config"),
            h.ListConfigs,
        )
        config.PUT("/:namespace/:key",
            middleware.RequirePermission("write", "config"),
            h.UpdateConfig,
        )
        config.DELETE("/:namespace/:key",
            middleware.RequirePermission("admin", "config"),
            h.DeleteConfig,
        )
    }

    // サービス向け設定一括取得
    services := v1.Group("/config/services")
    services.Use(middleware.RequireBearerToken())
    {
        services.GET("/:service_name", h.GetServiceConfig)
    }
}

func (h *RESTHandler) GetConfig(c *gin.Context) {
    namespace := c.Param("namespace")
    key := c.Param("key")

    entry, err := h.getConfigUC.Execute(c.Request.Context(), namespace, key)
    if err != nil {
        WriteError(c, http.StatusNotFound, "SYS_CONFIG_KEY_NOT_FOUND",
            "指定された設定キーが見つかりません")
        return
    }

    c.JSON(http.StatusOK, entry)
}

func (h *RESTHandler) UpdateConfig(c *gin.Context) {
    namespace := c.Param("namespace")
    key := c.Param("key")

    var req struct {
        Value       interface{} `json:"value" binding:"required"`
        Version     int         `json:"version" binding:"required"`
        Description string      `json:"description"`
    }
    if err := c.ShouldBindJSON(&req); err != nil {
        WriteError(c, http.StatusBadRequest, "SYS_CONFIG_VALIDATION_FAILED",
            "リクエストのバリデーションに失敗しました")
        return
    }

    // 更新者情報は JWT Claims から取得
    updatedBy := middleware.GetUserEmail(c)

    input := usecase.UpdateConfigInput{
        Namespace:   namespace,
        Key:         key,
        Value:       marshalValue(req.Value),
        Version:     req.Version,
        Description: req.Description,
        UpdatedBy:   updatedBy,
    }

    entry, err := h.updateConfigUC.Execute(c.Request.Context(), input)
    if err != nil {
        if err == usecase.ErrVersionConflict {
            WriteError(c, http.StatusConflict, "SYS_CONFIG_VERSION_CONFLICT",
                "設定値が他のユーザーによって更新されています。最新のバージョンを取得してください")
            return
        }
        WriteError(c, http.StatusInternalServerError, "SYS_CONFIG_UPDATE_FAILED",
            "設定値の更新に失敗しました")
        return
    }

    c.JSON(http.StatusOK, entry)
}

func (h *RESTHandler) DeleteConfig(c *gin.Context) {
    namespace := c.Param("namespace")
    key := c.Param("key")

    if err := h.deleteConfigUC.Execute(c.Request.Context(), namespace, key, middleware.GetUserEmail(c)); err != nil {
        WriteError(c, http.StatusNotFound, "SYS_CONFIG_KEY_NOT_FOUND",
            "指定された設定キーが見つかりません")
        return
    }

    c.Status(http.StatusNoContent)
}

// ... 他のハンドラーメソッドも同様のパターンで実装
```

### gRPC ハンドラー（Go）

```go
// internal/adapter/handler/grpc_handler.go
package handler

import (
    "context"

    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/status"

    pb "github.com/k1s0/regions/system/server/go/config/api/proto/k1s0/system/config/v1"
    "github.com/k1s0/regions/system/server/go/config/internal/usecase"
)

type GRPCHandler struct {
    pb.UnimplementedConfigServiceServer
    getConfigUC        *usecase.GetConfigUseCase
    listConfigsUC      *usecase.ListConfigsUseCase
    getServiceConfigUC *usecase.GetServiceConfigUseCase
}

func NewGRPCHandler(
    getConfigUC *usecase.GetConfigUseCase,
    listConfigsUC *usecase.ListConfigsUseCase,
    getServiceConfigUC *usecase.GetServiceConfigUseCase,
) *GRPCHandler {
    return &GRPCHandler{
        getConfigUC:        getConfigUC,
        listConfigsUC:      listConfigsUC,
        getServiceConfigUC: getServiceConfigUC,
    }
}

func (h *GRPCHandler) Register(s *grpc.Server) {
    pb.RegisterConfigServiceServer(s, h)
}

func (h *GRPCHandler) GetConfig(
    ctx context.Context, req *pb.GetConfigRequest,
) (*pb.GetConfigResponse, error) {
    entry, err := h.getConfigUC.Execute(ctx, req.Namespace, req.Key)
    if err != nil {
        return nil, status.Error(codes.NotFound, "config entry not found")
    }

    return &pb.GetConfigResponse{
        Entry: toProtoConfigEntry(entry),
    }, nil
}

func (h *GRPCHandler) ListConfigs(
    ctx context.Context, req *pb.ListConfigsRequest,
) (*pb.ListConfigsResponse, error) {
    entries, total, err := h.listConfigsUC.Execute(ctx, req.Namespace, usecase.ListConfigsInput{
        Search:   req.Search,
        Page:     int(req.Pagination.Page),
        PageSize: int(req.Pagination.PageSize),
    })
    if err != nil {
        return nil, status.Error(codes.Internal, "failed to list configs")
    }

    protoEntries := make([]*pb.ConfigEntry, len(entries))
    for i, e := range entries {
        protoEntries[i] = toProtoConfigEntry(e)
    }

    return &pb.ListConfigsResponse{
        Entries: protoEntries,
        Pagination: &pb.PaginationResult{
            TotalCount: int32(total),
        },
    }, nil
}

func (h *GRPCHandler) GetServiceConfig(
    ctx context.Context, req *pb.GetServiceConfigRequest,
) (*pb.GetServiceConfigResponse, error) {
    entries, err := h.getServiceConfigUC.Execute(ctx, req.ServiceName)
    if err != nil {
        return nil, status.Error(codes.NotFound, "service config not found")
    }

    protoEntries := make([]*pb.ConfigEntry, len(entries))
    for i, e := range entries {
        protoEntries[i] = toProtoConfigEntry(e)
    }

    return &pb.GetServiceConfigResponse{
        ServiceName: req.ServiceName,
        Entries:     protoEntries,
    }, nil
}
```

---

## Rust 実装 (regions/system/server/rust/config/)

### ディレクトリ構成

```
regions/system/server/rust/config/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── config_entry.rs          # ConfigEntry エンティティ
│   │   │   └── config_change_log.rs     # ConfigChangeLog エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── config_repository.rs     # ConfigRepository トレイト
│   │   │   └── config_change_log_repository.rs  # ConfigChangeLogRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── config_domain_service.rs # namespace バリデーション・バージョン検証
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── get_config.rs
│   │   ├── list_configs.rs
│   │   ├── update_config.rs
│   │   ├── delete_config.rs
│   │   └── get_service_config.rs
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── rest_handler.rs          # axum REST ハンドラー
│   │   │   ├── grpc_handler.rs          # tonic gRPC ハンドラー
│   │   │   └── error.rs                 # エラーレスポンス
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── response.rs
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infra/
│       ├── mod.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── logger.rs
│       ├── persistence/
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   ├── config_repository.rs
│       │   └── config_change_log_repository.rs
│       ├── cache/
│       │   ├── mod.rs
│       │   └── config_cache.rs          # インメモリキャッシュ
│       └── messaging/
│           ├── mod.rs
│           └── producer.rs              # Kafka プロデューサー
├── api/
│   └── proto/
│       └── k1s0/system/config/v1/
│           └── config.proto
├── migrations/
│   ├── 001_create_config_entries.sql
│   └── 002_create_config_change_logs.sql
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── build.rs                             # tonic-build（proto コンパイル）
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
└── README.md
```

### Cargo.toml

```toml
[package]
name = "config-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web フレームワーク
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
hyper = { version = "1", features = ["full"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# OpenTelemetry
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.28"

# キャッシュ
moka = { version = "0.12", features = ["future"] }

# ユーティリティ
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
validator = { version = "0.18", features = ["derive"] }

# メトリクス
prometheus = "0.13"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
testcontainers = "0.23"

[build-dependencies]
tonic-build = "0.12"
```

### build.rs

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["api/proto/k1s0/system/config/v1/config.proto"],
            &["api/proto/", "../../../../../../api/proto/"],
        )?;
    Ok(())
}
```

### src/main.rs

```rust
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tonic::transport::Server as TonicServer;
use tracing::info;

mod adapter;
mod domain;
mod infra;
mod usecase;

use adapter::handler::{grpc_handler, rest_handler};
use domain::service::ConfigDomainService;
use infra::cache::ConfigCache;
use infra::config::Config;
use infra::messaging::KafkaProducer;
use infra::persistence;
use usecase::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Config ---
    let cfg = Config::load("config/config.yaml")?;
    cfg.validate()?;

    // --- Logger ---
    infra::config::init_logger(&cfg.app.environment);

    // --- OpenTelemetry ---
    let _tracer = infra::config::init_tracer(&cfg.app.name)?;

    // --- Database ---
    let pool = persistence::connect(&cfg.database).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // --- Kafka ---
    let producer = Arc::new(KafkaProducer::new(&cfg.kafka)?);

    // --- Cache ---
    let config_cache = Arc::new(ConfigCache::new(
        cfg.config_server.cache.ttl_seconds,
        cfg.config_server.cache.max_entries,
    ));

    // --- DI ---
    let config_repo = Arc::new(persistence::ConfigRepositoryImpl::new(pool.clone()));
    let change_log_repo = Arc::new(persistence::ConfigChangeLogRepositoryImpl::new(pool.clone()));
    let config_domain_svc = Arc::new(ConfigDomainService::new());

    let get_config_uc = GetConfigUseCase::new(config_repo.clone(), config_cache.clone());
    let list_configs_uc = ListConfigsUseCase::new(config_repo.clone());
    let update_config_uc = UpdateConfigUseCase::new(
        config_repo.clone(),
        change_log_repo.clone(),
        config_domain_svc.clone(),
        config_cache.clone(),
        producer.clone(),
    );
    let delete_config_uc = DeleteConfigUseCase::new(
        config_repo.clone(),
        change_log_repo.clone(),
        config_cache.clone(),
        producer.clone(),
    );
    let get_service_config_uc = GetServiceConfigUseCase::new(
        config_repo.clone(), config_cache.clone(),
    );

    let app_state = AppState {
        get_config_uc: Arc::new(get_config_uc),
        list_configs_uc: Arc::new(list_configs_uc),
        update_config_uc: Arc::new(update_config_uc),
        delete_config_uc: Arc::new(delete_config_uc),
        get_service_config_uc: Arc::new(get_service_config_uc),
        pool: pool.clone(),
        producer: producer.clone(),
    };

    // --- REST Server (axum) ---
    let rest_app = rest_handler::router(app_state);
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));

    let rest_handle = tokio::spawn(async move {
        info!("REST server starting on {}", rest_addr);
        let listener = tokio::net::TcpListener::bind(rest_addr).await.unwrap();
        axum::serve(listener, rest_app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    });

    // --- gRPC Server (tonic) ---
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], cfg.grpc.port));
    let grpc_service = grpc_handler::ConfigServiceImpl::new(
        Arc::new(GetConfigUseCase::new(config_repo.clone(), config_cache.clone())),
        Arc::new(ListConfigsUseCase::new(config_repo.clone())),
        Arc::new(GetServiceConfigUseCase::new(config_repo, config_cache)),
    );

    let grpc_handle = tokio::spawn(async move {
        info!("gRPC server starting on {}", grpc_addr);
        TonicServer::builder()
            .add_service(grpc_handler::config_service_server(grpc_service))
            .serve_with_shutdown(grpc_addr, shutdown_signal())
            .await
            .unwrap();
    });

    tokio::try_join!(rest_handle, grpc_handle)?;
    info!("servers exited");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}
```

### ドメインモデル（Rust）

```rust
// src/domain/model/config_entry.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConfigEntry {
    pub id: Uuid,
    pub namespace: String,
    pub key: String,
    #[sqlx(json)]
    pub value: serde_json::Value,
    pub version: i32,
    pub description: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

```rust
// src/domain/model/config_change_log.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConfigChangeLog {
    pub id: Uuid,
    pub config_entry_id: Uuid,
    pub namespace: String,
    pub key: String,
    #[sqlx(json)]
    pub old_value: Option<serde_json::Value>,
    #[sqlx(json)]
    pub new_value: Option<serde_json::Value>,
    pub old_version: i32,
    pub new_version: i32,
    pub change_type: String, // CREATED, UPDATED, DELETED
    pub changed_by: String,
    pub changed_at: DateTime<Utc>,
}
```

### リポジトリトレイト（Rust）

```rust
// src/domain/repository/config_repository.rs
use async_trait::async_trait;

use crate::domain::model::ConfigEntry;

#[derive(Debug, Clone)]
pub struct ListParams {
    pub search: Option<String>,
    pub page: i32,
    pub page_size: i32,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>>;

    async fn list_by_namespace(
        &self,
        namespace: &str,
        params: &ListParams,
    ) -> anyhow::Result<(Vec<ConfigEntry>, i64)>;

    async fn list_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<Vec<ConfigEntry>>;

    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<()>;
    async fn update(&self, entry: &ConfigEntry) -> anyhow::Result<()>;
    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<()>;
}
```

```rust
// src/domain/repository/config_change_log_repository.rs
use async_trait::async_trait;

use crate::domain::model::ConfigChangeLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigChangeLogRepository: Send + Sync {
    async fn create(&self, log: &ConfigChangeLog) -> anyhow::Result<()>;
    async fn list_by_config_entry_id(
        &self,
        config_entry_id: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<ConfigChangeLog>, i64)>;
}
```

### ユースケース（Rust）

```rust
// src/usecase/get_config.rs
use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::ConfigEntry;
use crate::domain::repository::ConfigRepository;
use crate::infra::cache::ConfigCache;

pub struct GetConfigUseCase {
    repo: Arc<dyn ConfigRepository>,
    cache: Arc<ConfigCache>,
}

impl GetConfigUseCase {
    pub fn new(repo: Arc<dyn ConfigRepository>, cache: Arc<ConfigCache>) -> Self {
        Self { repo, cache }
    }

    #[instrument(skip(self), fields(service = "config-server"))]
    pub async fn execute(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<ConfigEntry, ConfigError> {
        // キャッシュから取得を試みる
        let cache_key = format!("{}:{}", namespace, key);
        if let Some(entry) = self.cache.get(&cache_key).await {
            return Ok(entry);
        }

        // DB から取得
        let entry = self
            .repo
            .find_by_namespace_and_key(namespace, key)
            .await
            .map_err(|_| ConfigError::InternalError)?
            .ok_or(ConfigError::KeyNotFound)?;

        // キャッシュに格納
        self.cache.set(&cache_key, &entry).await;

        Ok(entry)
    }
}
```

```rust
// src/usecase/update_config.rs
use std::sync::Arc;

use chrono::Utc;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::model::{ConfigChangeLog, ConfigEntry};
use crate::domain::repository::{ConfigChangeLogRepository, ConfigRepository};
use crate::domain::service::ConfigDomainService;
use crate::infra::cache::ConfigCache;
use crate::infra::messaging::KafkaProducer;

pub struct UpdateConfigUseCase {
    repo: Arc<dyn ConfigRepository>,
    change_log_repo: Arc<dyn ConfigChangeLogRepository>,
    domain_svc: Arc<ConfigDomainService>,
    cache: Arc<ConfigCache>,
    producer: Arc<KafkaProducer>,
}

impl UpdateConfigUseCase {
    pub fn new(
        repo: Arc<dyn ConfigRepository>,
        change_log_repo: Arc<dyn ConfigChangeLogRepository>,
        domain_svc: Arc<ConfigDomainService>,
        cache: Arc<ConfigCache>,
        producer: Arc<KafkaProducer>,
    ) -> Self {
        Self {
            repo,
            change_log_repo,
            domain_svc,
            cache,
            producer,
        }
    }

    #[instrument(skip(self, input), fields(service = "config-server"))]
    pub async fn execute(
        &self,
        input: UpdateConfigInput,
    ) -> Result<ConfigEntry, ConfigError> {
        // namespace バリデーション
        self.domain_svc
            .validate_namespace(&input.namespace)
            .map_err(|_| ConfigError::InvalidNamespace)?;

        // 現在の設定値を取得
        let mut current = self
            .repo
            .find_by_namespace_and_key(&input.namespace, &input.key)
            .await
            .map_err(|_| ConfigError::InternalError)?
            .ok_or(ConfigError::KeyNotFound)?;

        // バージョン検証（楽観的排他制御）
        self.domain_svc
            .validate_version(current.version, input.version)
            .map_err(|_| ConfigError::VersionConflict)?;

        // 更新
        let old_value = current.value.clone();
        current.value = input.value;
        current.version += 1;
        current.description = input.description;
        current.updated_by = input.updated_by.clone();
        current.updated_at = Utc::now();

        self.repo
            .update(&current)
            .await
            .map_err(|_| ConfigError::InternalError)?;

        // 変更ログ記録
        let change_log = ConfigChangeLog {
            id: Uuid::new_v4(),
            config_entry_id: current.id,
            namespace: current.namespace.clone(),
            key: current.key.clone(),
            old_value: Some(old_value),
            new_value: Some(current.value.clone()),
            old_version: input.version,
            new_version: current.version,
            change_type: "UPDATED".to_string(),
            changed_by: input.updated_by,
            changed_at: current.updated_at,
        };
        let _ = self.change_log_repo.create(&change_log).await;

        // キャッシュ無効化
        let cache_key = format!("{}:{}", current.namespace, current.key);
        self.cache.invalidate(&cache_key).await;

        // Kafka 通知
        self.producer.publish_config_changed(&change_log).await;

        Ok(current)
    }
}

pub struct UpdateConfigInput {
    pub namespace: String,
    pub key: String,
    pub value: serde_json::Value,
    pub version: i32,
    pub description: String,
    pub updated_by: String,
}
```

### REST ハンドラー（Rust）

```rust
// src/adapter/handler/rest_handler.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::adapter::handler::error::ErrorResponse;
use crate::adapter::middleware;

pub fn router(state: AppState) -> Router {
    Router::new()
        // ヘルスチェック
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        // 設定値管理
        .route(
            "/api/v1/config/:namespace/:key",
            get(get_config)
                .layer(middleware::require_permission("read", "config"))
                .put(update_config)
                .layer(middleware::require_permission("write", "config"))
                .delete(delete_config)
                .layer(middleware::require_permission("admin", "config")),
        )
        .route(
            "/api/v1/config/:namespace",
            get(list_configs).layer(middleware::require_permission("read", "config")),
        )
        // サービス向け設定一括取得
        .route(
            "/api/v1/config/services/:service_name",
            get(get_service_config).layer(middleware::require_bearer_token()),
        )
        .with_state(state)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    let kafka_ok = state.producer.healthy().await.is_ok();

    if db_ok && kafka_ok {
        Ok(Json(serde_json::json!({
            "status": "ready",
            "checks": {"database": "ok", "kafka": "ok"}
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn get_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let entry = state
        .get_config_uc
        .execute(&namespace, &key)
        .await
        .map_err(|_| {
            ErrorResponse::not_found(
                "SYS_CONFIG_KEY_NOT_FOUND",
                "指定された設定キーが見つかりません",
            )
        })?;

    Ok(Json(serde_json::to_value(entry).unwrap()))
}

#[derive(Deserialize)]
struct UpdateConfigRequest {
    value: serde_json::Value,
    version: i32,
    description: Option<String>,
}

async fn update_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let updated_by = middleware::get_user_email_from_context();

    let input = UpdateConfigInput {
        namespace,
        key,
        value: req.value,
        version: req.version,
        description: req.description.unwrap_or_default(),
        updated_by,
    };

    let entry = state
        .update_config_uc
        .execute(input)
        .await
        .map_err(|e| match e {
            ConfigError::VersionConflict => ErrorResponse::conflict(
                "SYS_CONFIG_VERSION_CONFLICT",
                "設定値が他のユーザーによって更新されています。最新のバージョンを取得してください",
            ),
            ConfigError::KeyNotFound => ErrorResponse::not_found(
                "SYS_CONFIG_KEY_NOT_FOUND",
                "指定された設定キーが見つかりません",
            ),
            _ => ErrorResponse::internal(
                "SYS_CONFIG_UPDATE_FAILED",
                "設定値の更新に失敗しました",
            ),
        })?;

    Ok(Json(serde_json::to_value(entry).unwrap()))
}

async fn delete_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Result<StatusCode, ErrorResponse> {
    let deleted_by = middleware::get_user_email_from_context();

    state
        .delete_config_uc
        .execute(&namespace, &key, &deleted_by)
        .await
        .map_err(|_| {
            ErrorResponse::not_found(
                "SYS_CONFIG_KEY_NOT_FOUND",
                "指定された設定キーが見つかりません",
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ... 他のハンドラーも同様のパターンで実装
```

### gRPC ハンドラー（Rust）

```rust
// src/adapter/handler/grpc_handler.rs
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("k1s0.system.config.v1");
}

use proto::config_service_server::{ConfigService, ConfigServiceServer};
use proto::*;

pub struct ConfigServiceImpl {
    get_config_uc: Arc<GetConfigUseCase>,
    list_configs_uc: Arc<ListConfigsUseCase>,
    get_service_config_uc: Arc<GetServiceConfigUseCase>,
}

impl ConfigServiceImpl {
    pub fn new(
        get_config_uc: Arc<GetConfigUseCase>,
        list_configs_uc: Arc<ListConfigsUseCase>,
        get_service_config_uc: Arc<GetServiceConfigUseCase>,
    ) -> Self {
        Self {
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
        }
    }
}

pub fn config_service_server(svc: ConfigServiceImpl) -> ConfigServiceServer<ConfigServiceImpl> {
    ConfigServiceServer::new(svc)
}

#[tonic::async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        let req = request.into_inner();

        let entry = self
            .get_config_uc
            .execute(&req.namespace, &req.key)
            .await
            .map_err(|_| Status::not_found("config entry not found"))?;

        Ok(Response::new(GetConfigResponse {
            entry: Some(entry.into()),
        }))
    }

    async fn list_configs(
        &self,
        request: Request<ListConfigsRequest>,
    ) -> Result<Response<ListConfigsResponse>, Status> {
        let req = request.into_inner();

        let (entries, total) = self
            .list_configs_uc
            .execute(
                &req.namespace,
                ListConfigsInput {
                    search: if req.search.is_empty() { None } else { Some(req.search) },
                    page: req.pagination.as_ref().map(|p| p.page).unwrap_or(1),
                    page_size: req.pagination.as_ref().map(|p| p.page_size).unwrap_or(20),
                },
            )
            .await
            .map_err(|_| Status::internal("failed to list configs"))?;

        let proto_entries: Vec<proto::ConfigEntry> =
            entries.into_iter().map(|e| e.into()).collect();

        Ok(Response::new(ListConfigsResponse {
            entries: proto_entries,
            pagination: Some(proto::PaginationResult {
                total_count: total as i32,
            }),
        }))
    }

    async fn get_service_config(
        &self,
        request: Request<GetServiceConfigRequest>,
    ) -> Result<Response<GetServiceConfigResponse>, Status> {
        let req = request.into_inner();

        let entries = self
            .get_service_config_uc
            .execute(&req.service_name)
            .await
            .map_err(|_| Status::not_found("service config not found"))?;

        let proto_entries: Vec<proto::ConfigEntry> =
            entries.into_iter().map(|e| e.into()).collect();

        Ok(Response::new(GetServiceConfigResponse {
            service_name: req.service_name,
            entries: proto_entries,
        }))
    }

    type WatchConfigStream = tokio_stream::wrappers::ReceiverStream<
        Result<ConfigChangeEvent, Status>,
    >;

    async fn watch_config(
        &self,
        request: Request<WatchConfigRequest>,
    ) -> Result<Response<Self::WatchConfigStream>, Status> {
        let req = request.into_inner();
        let namespaces = req.namespaces;

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // Kafka Consumer からのイベントをストリームとして配信
        // ...（実装省略）

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
```

---

## config.yaml

[config設計.md](config設計.md) のスキーマを拡張した設定管理サーバー固有の設定。

```yaml
# config/config.yaml
app:
  name: "config-server"
  version: "0.1.0"
  tier: "system"
  environment: "dev"

server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"

grpc:
  port: 50051
  max_recv_msg_size: 4194304  # 4MB

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "config_db"
  user: "app"
  password: ""               # Vault パス: secret/data/k1s0/system/config/database キー: password
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "config-server.default"
  security_protocol: "PLAINTEXT"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""             # Vault パス: secret/data/k1s0/system/kafka/sasl キー: username
    password: ""             # Vault パス: secret/data/k1s0/system/kafka/sasl キー: password
  topics:
    publish:
      - "k1s0.system.config.changed.v1"
    subscribe: []

observability:
  log:
    level: "debug"
    format: "json"
  trace:
    enabled: true
    endpoint: "jaeger.observability.svc.cluster.local:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"

# 設定管理サーバー固有設定
config_server:
  # インメモリキャッシュ
  cache:
    ttl: "60s"               # キャッシュの TTL（デフォルト 60 秒）
    max_entries: 10000        # キャッシュの最大エントリ数
    refresh_on_miss: true     # キャッシュミス時にバックグラウンドリフレッシュ
  # 監査ログ
  audit:
    kafka_enabled: true       # Kafka への非同期配信を有効化
    retention_days: 365       # DB 内の保持日数
  # namespace バリデーション
  namespace:
    allowed_tiers:
      - "system"
      - "business"
      - "service"
    max_depth: 4              # namespace の最大階層数
```

### 設定の読み込み実装

#### Go

```go
// internal/infra/config/config.go
package config

import (
    "fmt"
    "os"
    "time"

    "github.com/go-playground/validator/v10"
    "gopkg.in/yaml.v3"
)

type Config struct {
    App           AppConfig           `yaml:"app"`
    Server        ServerConfig        `yaml:"server"`
    GRPC          GRPCConfig          `yaml:"grpc"`
    Database      DatabaseConfig      `yaml:"database"`
    Kafka         KafkaConfig         `yaml:"kafka"`
    Observability ObservabilityConfig `yaml:"observability"`
    ConfigServer  ConfigServerConfig  `yaml:"config_server"`
}

type ServerConfig struct {
    Host            string        `yaml:"host"`
    Port            int           `yaml:"port" validate:"required,min=1,max=65535"`
    ReadTimeout     time.Duration `yaml:"read_timeout"`
    WriteTimeout    time.Duration `yaml:"write_timeout"`
    ShutdownTimeout time.Duration `yaml:"shutdown_timeout"`
}

type ConfigServerConfig struct {
    Cache     CacheConfig     `yaml:"cache"`
    Audit     AuditConfig     `yaml:"audit"`
    Namespace NamespaceConfig `yaml:"namespace"`
}

type CacheConfig struct {
    TTL           string `yaml:"ttl"`
    MaxEntries    int    `yaml:"max_entries"`
    RefreshOnMiss bool   `yaml:"refresh_on_miss"`
}

type AuditConfig struct {
    KafkaEnabled  bool `yaml:"kafka_enabled"`
    RetentionDays int  `yaml:"retention_days"`
}

type NamespaceConfig struct {
    AllowedTiers []string `yaml:"allowed_tiers"`
    MaxDepth     int      `yaml:"max_depth"`
}

func Load(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("failed to read config: %w", err)
    }
    var cfg Config
    if err := yaml.Unmarshal(data, &cfg); err != nil {
        return nil, fmt.Errorf("failed to parse config: %w", err)
    }
    return &cfg, nil
}

func (c *Config) Validate() error {
    validate := validator.New()
    return validate.Struct(c)
}
```

#### Rust

```rust
// src/infra/config/mod.rs
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub grpc: GrpcConfig,
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub observability: ObservabilityConfig,
    pub config_server: ConfigServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub read_timeout: String,
    pub write_timeout: String,
    pub shutdown_timeout: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServerConfig {
    pub cache: CacheConfig,
    pub audit: AuditServerConfig,
    pub namespace: NamespaceConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub ttl: String,
    pub max_entries: u64,
    pub refresh_on_miss: bool,

    #[serde(skip)]
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditServerConfig {
    pub kafka_enabled: bool,
    pub retention_days: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceConfig {
    pub allowed_tiers: Vec<String>,
    pub max_depth: usize,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.app.name.is_empty() {
            anyhow::bail!("app.name is required");
        }
        if self.server.port == 0 {
            anyhow::bail!("server.port must be > 0");
        }
        if self.config_server.cache.max_entries == 0 {
            anyhow::bail!("config_server.cache.max_entries must be > 0");
        }
        if self.config_server.namespace.allowed_tiers.is_empty() {
            anyhow::bail!("config_server.namespace.allowed_tiers must not be empty");
        }
        Ok(())
    }
}
```

---

## キャッシュ戦略

設定値の取得は高頻度で呼び出されるため、インメモリキャッシュによるレイテンシ削減を行う。

### キャッシュ方針

| 項目 | 値 |
| --- | --- |
| キャッシュ方式 | インメモリ（Go: ristretto, Rust: moka） |
| TTL | 設定可能（デフォルト 60 秒） |
| 最大エントリ数 | 設定可能（デフォルト 10,000） |
| キャッシュキー | `{namespace}:{key}` 形式 |
| 無効化タイミング | PUT / DELETE 実行時に即座に無効化 |
| キャッシュミス | DB から取得後にキャッシュに格納 |

### キャッシュ無効化フロー

```
1. PUT /api/v1/config/:namespace/:key が呼ばれる
2. DB を更新（楽観的排他制御によるバージョン検証）
3. config_change_logs テーブルに変更ログを記録
4. インメモリキャッシュの該当エントリを無効化
5. Kafka トピック k1s0.system.config.changed.v1 にイベントを発行
6. 他サービスは Kafka イベントを受信してローカルキャッシュを無効化
```

### Go キャッシュ実装例

```go
// internal/infra/cache/config_cache.go
package cache

import (
    "time"

    "github.com/dgraph-io/ristretto"

    "github.com/k1s0/regions/system/server/go/config/internal/domain/model"
)

type ConfigCache struct {
    cache *ristretto.Cache
    ttl   time.Duration
}

func NewConfigCache(ttl string, maxEntries int) *ConfigCache {
    d, _ := time.ParseDuration(ttl)

    cache, _ := ristretto.NewCache(&ristretto.Config{
        NumCounters: int64(maxEntries) * 10,
        MaxCost:     int64(maxEntries),
        BufferItems: 64,
    })

    return &ConfigCache{
        cache: cache,
        ttl:   d,
    }
}

func (c *ConfigCache) Get(key string) (*model.ConfigEntry, bool) {
    val, found := c.cache.Get(key)
    if !found {
        return nil, false
    }
    entry, ok := val.(*model.ConfigEntry)
    return entry, ok
}

func (c *ConfigCache) Set(key string, entry *model.ConfigEntry) {
    c.cache.SetWithTTL(key, entry, 1, c.ttl)
}

func (c *ConfigCache) Delete(key string) {
    c.cache.Del(key)
}
```

### Rust キャッシュ実装例

```rust
// src/infra/cache/config_cache.rs
use moka::future::Cache;
use std::time::Duration;

use crate::domain::model::ConfigEntry;

pub struct ConfigCache {
    cache: Cache<String, ConfigEntry>,
}

impl ConfigCache {
    pub fn new(ttl_seconds: u64, max_entries: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self { cache }
    }

    pub async fn get(&self, key: &str) -> Option<ConfigEntry> {
        self.cache.get(key).await
    }

    pub async fn set(&self, key: &str, entry: &ConfigEntry) {
        self.cache.insert(key.to_string(), entry.clone()).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }
}
```

---

## データベースマイグレーション

設定値と変更ログの2テーブルを PostgreSQL（config-db）に格納する。

```sql
-- migrations/001_create_config_entries.sql

CREATE TABLE IF NOT EXISTS config_entries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    namespace   VARCHAR(255) NOT NULL,
    key         VARCHAR(255) NOT NULL,
    value       JSONB        NOT NULL,
    version     INTEGER      NOT NULL DEFAULT 1,
    description TEXT         NOT NULL DEFAULT '',
    created_by  VARCHAR(255) NOT NULL,
    updated_by  VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_config_entries_namespace_key UNIQUE (namespace, key)
);

-- 検索用インデックス
CREATE INDEX idx_config_entries_namespace ON config_entries (namespace);
CREATE INDEX idx_config_entries_key ON config_entries (key);
CREATE INDEX idx_config_entries_updated_at ON config_entries (updated_at DESC);

-- サービス名検索用（namespace の第2階層がサービス名に対応）
-- 例: system.auth.database → auth がサービス名
CREATE INDEX idx_config_entries_namespace_prefix ON config_entries USING btree (namespace varchar_pattern_ops);

COMMENT ON TABLE config_entries IS '設定値エントリ。namespace.key の一意制約で管理。';
COMMENT ON COLUMN config_entries.namespace IS 'Tier.Service.Section 形式の名前空間（例: system.auth.database）';
COMMENT ON COLUMN config_entries.value IS 'JSONB 形式の設定値。string, number, boolean, object を格納可能';
COMMENT ON COLUMN config_entries.version IS '楽観的排他制御用のバージョン番号。更新のたびにインクリメント';
```

```sql
-- migrations/002_create_config_change_logs.sql

CREATE TABLE IF NOT EXISTS config_change_logs (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    config_entry_id  UUID         NOT NULL,
    namespace        VARCHAR(255) NOT NULL,
    key              VARCHAR(255) NOT NULL,
    old_value        JSONB,
    new_value        JSONB,
    old_version      INTEGER      NOT NULL DEFAULT 0,
    new_version      INTEGER      NOT NULL,
    change_type      VARCHAR(20)  NOT NULL CHECK (change_type IN ('CREATED', 'UPDATED', 'DELETED')),
    changed_by       VARCHAR(255) NOT NULL,
    changed_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- 検索用インデックス
CREATE INDEX idx_config_change_logs_config_entry_id ON config_change_logs (config_entry_id);
CREATE INDEX idx_config_change_logs_namespace ON config_change_logs (namespace);
CREATE INDEX idx_config_change_logs_changed_at ON config_change_logs (changed_at DESC);
CREATE INDEX idx_config_change_logs_changed_by ON config_change_logs (changed_by);
CREATE INDEX idx_config_change_logs_change_type ON config_change_logs (change_type);

-- 複合インデックス（設定エントリ + 日時範囲の検索最適化）
CREATE INDEX idx_config_change_logs_entry_changed ON config_change_logs (config_entry_id, changed_at DESC);

-- パーティショニング（月単位）は運用フェーズで検討
COMMENT ON TABLE config_change_logs IS '設定変更の監査ログ。全ての CRUD 操作を記録。保持期間は 1 年間（可観測性設計.md 参照）';
```

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | Go | Rust |
| --- | --- | --- | --- |
| domain/service | 単体テスト | `testify/assert` | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `gomock` | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `httptest` + `testify` | `axum::test` + `tokio::test` |
| infra/persistence | 統合テスト（DB） | `testcontainers-go` | `testcontainers` |
| infra/cache | 単体テスト | `testify` | `tokio::test` |

### Go テスト例

```go
// internal/usecase/get_config_test.go
package usecase

import (
    "context"
    "encoding/json"
    "testing"

    "github.com/stretchr/testify/assert"
    "go.uber.org/mock/gomock"

    "github.com/k1s0/regions/system/server/go/config/internal/domain/model"
    "github.com/k1s0/regions/system/server/go/config/internal/infra/cache"
)

func TestGetConfig_CacheHit(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()

    mockRepo := repository.NewMockConfigRepository(ctrl)
    // キャッシュヒット時は DB 呼び出しなし
    mockRepo.EXPECT().FindByNamespaceAndKey(gomock.Any(), gomock.Any(), gomock.Any()).Times(0)

    configCache := cache.NewConfigCache("60s", 1000)
    entry := &model.ConfigEntry{
        Namespace: "system.auth.database",
        Key:       "max_connections",
        Value:     json.RawMessage(`25`),
        Version:   3,
    }
    configCache.Set("system.auth.database:max_connections", entry)

    uc := NewGetConfigUseCase(mockRepo, configCache)

    result, err := uc.Execute(context.Background(), "system.auth.database", "max_connections")
    assert.NoError(t, err)
    assert.Equal(t, "max_connections", result.Key)
    assert.Equal(t, 3, result.Version)
}

func TestGetConfig_CacheMiss(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()

    entry := &model.ConfigEntry{
        Namespace: "system.auth.database",
        Key:       "max_connections",
        Value:     json.RawMessage(`25`),
        Version:   3,
    }

    mockRepo := repository.NewMockConfigRepository(ctrl)
    mockRepo.EXPECT().
        FindByNamespaceAndKey(gomock.Any(), "system.auth.database", "max_connections").
        Return(entry, nil)

    configCache := cache.NewConfigCache("60s", 1000)

    uc := NewGetConfigUseCase(mockRepo, configCache)

    result, err := uc.Execute(context.Background(), "system.auth.database", "max_connections")
    assert.NoError(t, err)
    assert.Equal(t, "max_connections", result.Key)

    // キャッシュに格納されていることを確認
    cached, ok := configCache.Get("system.auth.database:max_connections")
    assert.True(t, ok)
    assert.Equal(t, entry.Version, cached.Version)
}

func TestUpdateConfig_VersionConflict(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()

    current := &model.ConfigEntry{
        Namespace: "system.auth.database",
        Key:       "max_connections",
        Value:     json.RawMessage(`25`),
        Version:   4, // DB のバージョンは 4
    }

    mockRepo := repository.NewMockConfigRepository(ctrl)
    mockRepo.EXPECT().
        FindByNamespaceAndKey(gomock.Any(), "system.auth.database", "max_connections").
        Return(current, nil)

    mockChangeLogRepo := repository.NewMockConfigChangeLogRepository(ctrl)
    domainSvc := service.NewConfigDomainService()
    configCache := cache.NewConfigCache("60s", 1000)
    producer := messaging.NewMockProducer()

    uc := NewUpdateConfigUseCase(mockRepo, mockChangeLogRepo, domainSvc, configCache, producer)

    input := UpdateConfigInput{
        Namespace: "system.auth.database",
        Key:       "max_connections",
        Value:     json.RawMessage(`50`),
        Version:   3, // クライアントのバージョンは 3（古い）
        UpdatedBy: "operator@example.com",
    }

    _, err := uc.Execute(context.Background(), input)
    assert.ErrorIs(t, err, ErrVersionConflict)
}
```

### Rust テスト例

```rust
// src/usecase/get_config.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::MockConfigRepository;
    use crate::infra::cache::ConfigCache;

    #[tokio::test]
    async fn test_get_config_cache_hit() {
        let mut mock_repo = MockConfigRepository::new();
        // キャッシュヒット時は DB 呼び出しなし
        mock_repo.expect_find_by_namespace_and_key().times(0);

        let cache = Arc::new(ConfigCache::new(60, 1000));
        let entry = ConfigEntry {
            id: uuid::Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(25),
            version: 3,
            description: String::new(),
            created_by: "admin".to_string(),
            updated_by: "admin".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        cache.set("system.auth.database:max_connections", &entry).await;

        let uc = GetConfigUseCase::new(Arc::new(mock_repo), cache);

        let result = uc.execute("system.auth.database", "max_connections").await.unwrap();
        assert_eq!(result.key, "max_connections");
        assert_eq!(result.version, 3);
    }

    #[tokio::test]
    async fn test_get_config_cache_miss() {
        let mut mock_repo = MockConfigRepository::new();
        let entry = ConfigEntry {
            id: uuid::Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(25),
            version: 3,
            description: String::new(),
            created_by: "admin".to_string(),
            updated_by: "admin".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let entry_clone = entry.clone();

        mock_repo
            .expect_find_by_namespace_and_key()
            .with(eq("system.auth.database"), eq("max_connections"))
            .returning(move |_, _| Ok(Some(entry_clone.clone())));

        let cache = Arc::new(ConfigCache::new(60, 1000));
        let uc = GetConfigUseCase::new(Arc::new(mock_repo), cache.clone());

        let result = uc.execute("system.auth.database", "max_connections").await.unwrap();
        assert_eq!(result.key, "max_connections");

        // キャッシュに格納されていることを確認
        let cached = cache.get("system.auth.database:max_connections").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().version, 3);
    }
}
```

### testcontainers による DB 統合テスト

#### Go

```go
// internal/infra/persistence/config_repository_test.go
package persistence

import (
    "context"
    "encoding/json"
    "testing"

    "github.com/stretchr/testify/assert"
    "github.com/testcontainers/testcontainers-go"
    "github.com/testcontainers/testcontainers-go/modules/postgres"
)

func TestConfigRepository_CRUD(t *testing.T) {
    ctx := context.Background()

    pgContainer, err := postgres.Run(ctx, "postgres:16-alpine",
        postgres.WithDatabase("config_db_test"),
    )
    assert.NoError(t, err)
    defer pgContainer.Terminate(ctx)

    connStr, err := pgContainer.ConnectionString(ctx, "sslmode=disable")
    assert.NoError(t, err)

    db, err := NewDB(DatabaseConfig{/* connStr から設定 */})
    assert.NoError(t, err)
    defer db.Close()

    // マイグレーション実行
    runMigrations(db)

    repo := NewConfigRepository(db)

    // Create
    entry := &model.ConfigEntry{
        Namespace:   "system.auth.database",
        Key:         "max_connections",
        Value:       json.RawMessage(`25`),
        Version:     1,
        Description: "DB 最大接続数",
        CreatedBy:   "admin@example.com",
        UpdatedBy:   "admin@example.com",
    }
    err = repo.Create(ctx, entry)
    assert.NoError(t, err)

    // Read
    found, err := repo.FindByNamespaceAndKey(ctx, "system.auth.database", "max_connections")
    assert.NoError(t, err)
    assert.Equal(t, "max_connections", found.Key)
    assert.Equal(t, 1, found.Version)

    // Update
    found.Value = json.RawMessage(`50`)
    found.Version = 2
    err = repo.Update(ctx, found)
    assert.NoError(t, err)

    // Delete
    err = repo.Delete(ctx, "system.auth.database", "max_connections")
    assert.NoError(t, err)

    _, err = repo.FindByNamespaceAndKey(ctx, "system.auth.database", "max_connections")
    assert.Error(t, err)
}
```

#### Rust

```rust
// src/infra/persistence/config_repository_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::{runners::AsyncRunner, GenericImage};

    #[tokio::test]
    async fn test_config_repository_crud() {
        let container = GenericImage::new("postgres", "16-alpine")
            .with_env_var("POSTGRES_DB", "config_db_test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .start()
            .await
            .unwrap();

        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = sqlx::PgPool::connect(
            &format!("postgresql://postgres:test@localhost:{}/config_db_test", port),
        )
        .await
        .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = ConfigRepositoryImpl::new(pool);

        // Create
        let entry = ConfigEntry {
            id: uuid::Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(25),
            version: 1,
            description: "DB 最大接続数".to_string(),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        repo.create(&entry).await.unwrap();

        // Read
        let found = repo
            .find_by_namespace_and_key("system.auth.database", "max_connections")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.key, "max_connections");
        assert_eq!(found.version, 1);

        // Update
        let mut updated = found.clone();
        updated.value = serde_json::json!(50);
        updated.version = 2;
        repo.update(&updated).await.unwrap();

        // Delete
        repo.delete("system.auth.database", "max_connections")
            .await
            .unwrap();

        let deleted = repo
            .find_by_namespace_and_key("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(deleted.is_none());
    }
}
```

---

## デプロイ

### Dockerfile

[Dockerイメージ戦略.md](Dockerイメージ戦略.md) のテンプレートに従う。

#### Go

```dockerfile
# ---- Build ----
FROM golang:1.23-bookworm AS build
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /app ./cmd/

# ---- Runtime ----
FROM gcr.io/distroless/static-debian12
COPY --from=build /app /app
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/app"]
```

#### Rust

```dockerfile
# ---- Build ----
FROM rust:1.82-bookworm AS build
WORKDIR /src

# protoc のインストール（tonic-build に必要）
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

# ---- Runtime ----
FROM gcr.io/distroless/cc-debian12
COPY --from=build /src/target/release/config-server /app
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/app"]
```

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。設定管理サーバー固有の values は以下の通り。

```yaml
# values-config.yaml
app:
  name: config-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/config-server
  tag: "0.1.0"

service:
  ports:
    - name: http
      port: 80
      targetPort: 8080
    - name: grpc
      port: 50051
      targetPort: 50051

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 256Mi

# Vault Agent Injector
podAnnotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/role: "system"
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/config/database"
  vault.hashicorp.com/agent-inject-secret-kafka-sasl: "secret/data/k1s0/system/kafka/sasl"

# ヘルスチェック
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5

# ConfigMap マウント
configMap:
  name: config-server-config
  mountPath: /etc/app/config.yaml
```

### Kong ルーティング

[認証認可設計.md](認証認可設計.md) の Kong ルーティング設計に従い、設定管理サーバーを Kong に登録する。

```yaml
services:
  - name: config-v1
    url: http://config-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: config-v1-route
        paths:
          - /api/v1/config
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [認証認可設計.md](認証認可設計.md) -- Keycloak 設定・OAuth 2.0 フロー・RBAC 設計・Vault 戦略
- [API設計.md](API設計.md) -- REST / gRPC / GraphQL 設計・エラーレスポンス・バージョニング
- [config設計.md](config設計.md) -- config.yaml スキーマと環境別管理
- [可観測性設計.md](可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [メッセージング設計.md](メッセージング設計.md) -- Kafka トピック設計・監査イベント配信
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](コーディング規約.md) -- Linter・Formatter・命名規則
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャの詳細
- [system-server設計.md](system-server設計.md) -- 認証サーバー設計（同 tier の参考実装）
- [APIゲートウェイ設計.md](APIゲートウェイ設計.md) -- Kong 構成管理
- [サービスメッシュ設計.md](サービスメッシュ設計.md) -- Istio 設計・mTLS
