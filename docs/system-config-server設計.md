# system-config-server 設計

system tier の設定管理サーバー設計を定義する。全サービスに対して REST と gRPC で設定値を提供し、設定変更時の通知・監査ログ記録を行う。
Rust で実装する。

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

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC フレームワーク | tonic v0.12 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| キャッシュ | moka v0.12 |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
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
| GET | `/api/v1/config-schema` | 設定スキーマ一覧 | `sys_auditor` 以上 |
| POST | `/api/v1/config-schema` | 設定スキーマ作成 | `sys_operator` 以上 |
| GET | `/api/v1/config-schema/:name` | 設定スキーマ取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/config-schema/:name` | 設定スキーマ更新 | `sys_operator` 以上 |
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
infrastructure（DB接続・Kafka プロデューサー・キャッシュ・設定ローダー）
```

| レイヤー | パッケージ / モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `ConfigEntry`, `ConfigChangeLog` | エンティティ定義 |
| domain/repository | `ConfigRepository`, `ConfigChangeLogRepository` | リポジトリインターフェース / トレイト |
| domain/service | `ConfigDomainService` | ドメインサービス（namespace バリデーション・バージョン検証ロジック） |
| usecase | `GetConfigUsecase`, `ListConfigsUsecase`, `UpdateConfigUsecase`, `DeleteConfigUsecase`, `GetServiceConfigUsecase` | ユースケース |
| adapter/handler | REST ハンドラー, gRPC ハンドラー | プロトコル変換 |
| adapter/presenter | レスポンスフォーマット | ドメインモデル → API レスポンス変換 |
| adapter/gateway | （外部サービスなし） | - |
| infrastructure/persistence | PostgreSQL リポジトリ実装 | 設定値・変更ログの永続化 |
| infrastructure/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infrastructure/messaging | Kafka プロデューサー | 設定変更イベントの非同期配信 |
| infrastructure/cache | インメモリキャッシュ | 設定値のキャッシュ管理（TTL 制御） |

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
    │  domain/entity  │  │ domain/service │  │ domain/repository   │
    │  ConfigEntry,   │  │ ConfigDomain   │  │ ConfigRepository    │
    │  ConfigChange   │  │ Service        │  │ ConfigChangeLog     │
    │  Log            │  │                │  │ Repository          │
    └────────────────┘  └────────────────┘  │ (interface/trait)    │
                                            └──────────┬──────────┘
                                                       │
                    ┌──────────────────────────────────┼──────────────┐
                    │             infrastructure 層         │              │
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

## 詳細設計ドキュメント

実装・デプロイの詳細は以下の分割ドキュメントを参照。

- [system-config-server-実装設計.md](system-config-server-実装設計.md) -- Rust 実装詳細（Cargo.toml・ドメイン・リポジトリ・ユースケース・ハンドラー・config.yaml）
- [system-config-server-デプロイ設計.md](system-config-server-デプロイ設計.md) -- キャッシュ戦略・DB マイグレーション・テスト・Dockerfile・Helm values

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
