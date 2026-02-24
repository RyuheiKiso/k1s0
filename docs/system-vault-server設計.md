# system-vault-server 設計

system tier の Vault 統合サーバー設計を定義する。HashiCorp Vault からシークレットを取得し、全サービスへの安全な配布を仲介する。シークレットのローテーション時に Kafka でサービスに通知し、シークレットアクセスの監査ログを PostgreSQL に記録する。
Rust での実装を定義する。

## 概要

system tier の Vault Server は以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| シークレット取得 | Vault KV v2 / Dynamic Secrets からシークレットを取得して各サービスに提供する |
| シークレット一覧 | 権限範囲内のシークレットパス一覧取得 |
| シークレットローテーション | 手動または Vault リース期限に基づく自動ローテーション実行 |
| ローテーション通知 | シークレットローテーション時に Kafka `k1s0.system.vault.rotated.v1` でサービスに通知 |
| アクセス監査ログ | シークレットアクセス・ローテーションイベントを PostgreSQL に記録 |
| ヘルスモニタリング | Vault の接続状態・リース有効期限の監視 |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC | tonic v0.12 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| Vault クライアント | vaultrs v0.7 |
| OTel | opentelemetry v0.27 / k1s0-telemetry |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| キャッシュ | moka v0.12 |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/vault/` |

---

## 設計方針

[Vault設計.md](Vault設計.md) および [サービス間認証設計.md](サービス間認証設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| Vault 認証方式 | AppRole 認証（サービス用 role_id / secret_id）または Kubernetes Auth |
| シークレットキャッシュ | moka でシークレット値を TTL（リース期限の 80%）キャッシュ。ローテーション時にクリア |
| 直接アクセス禁止 | 各サービスは Vault に直接アクセスせず、本サーバーを経由する |
| アクセス制御 | 要求元サービスのサービスアカウント（SPIFFE ID）に基づくアクセス制御 |
| ローテーション自動化 | Vault Agent Sidecar と協調。本サーバーはローテーションイベントを Kafka で通知 |
| DB | PostgreSQL の `vault` スキーマ（監査ログのみ。シークレット値は DB 保存しない） |
| RBAC | `sys_admin`（全権限）/ `sys_operator`（ローテーション実行）/ `sys_auditor`（読み取り・監査ログ閲覧） |
| Kafka オプショナル | Kafka 未設定時もシークレット取得は動作する。ローテーション通知のみスキップ |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_VAULT_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/secrets/:path` | シークレット取得 | SPIFFE ID ベースの認可 |
| GET | `/api/v1/secrets/:path/metadata` | シークレットメタデータ取得 | SPIFFE ID ベースの認可 |
| GET | `/api/v1/secrets` | シークレット一覧（パスのみ） | `sys_auditor` 以上 |
| POST | `/api/v1/secrets/:path/rotate` | シークレットローテーション | `sys_operator` 以上 |
| GET | `/api/v1/audit/logs` | アクセス監査ログ | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/secrets/:path

指定パスのシークレットを Vault から取得する。キャッシュが有効な場合はキャッシュから返却する。アクセスは監査ログに記録される。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `version` | int | No | - | シークレットのバージョン（未指定時は最新） |

**レスポンス（200 OK）**

```json
{
  "data": {
    "username": "db_admin",
    "password": "s3cret-v4lue"
  },
  "version": 3,
  "created_at": "2026-02-23T10:00:00.000+00:00"
}
```

**レスポンス（403 Forbidden）**

```json
{
  "error": {
    "code": "SYS_VAULT_ACCESS_DENIED",
    "message": "service 'spiffe://k1s0/ns/default/sa/order-service' is not authorized to access 'secret/data/k1s0/system/auth/database'",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_VAULT_SECRET_NOT_FOUND",
    "message": "secret not found at path: secret/data/k1s0/system/nonexistent",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（502 Bad Gateway）**

```json
{
  "error": {
    "code": "SYS_VAULT_UPSTREAM_ERROR",
    "message": "failed to connect to Vault: connection refused",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/secrets/:path/metadata

シークレットのメタデータ（バージョン情報、リース期限等）を取得する。シークレット値は含まない。

**レスポンス（200 OK）**

```json
{
  "path": "secret/data/k1s0/system/auth/database",
  "current_version": 3,
  "oldest_version": 1,
  "created_at": "2026-01-15T08:00:00.000+00:00",
  "updated_at": "2026-02-23T10:00:00.000+00:00",
  "lease_duration": "768h"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_VAULT_SECRET_NOT_FOUND",
    "message": "secret metadata not found at path: secret/data/k1s0/system/nonexistent",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/secrets

権限範囲内のシークレットパス一覧を取得する。シークレット値は含まない。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `path_prefix` | string | No | - | パスプレフィックスフィルタ（例: `secret/data/k1s0/system/`） |

**レスポンス（200 OK）**

```json
{
  "paths": [
    "secret/data/k1s0/system/auth/database",
    "secret/data/k1s0/system/auth/jwt-signing-key",
    "secret/data/k1s0/system/config/database",
    "secret/data/k1s0/system/saga/database",
    "secret/data/k1s0/system/kafka/sasl"
  ]
}
```

#### POST /api/v1/secrets/:path/rotate

指定パスのシークレットをローテーションする。Vault の KV v2 に新バージョンを書き込み、キャッシュをクリアし、Kafka でローテーション通知を配信する。

**リクエスト**

```json
{
  "reason": "Scheduled quarterly rotation"
}
```

**レスポンス（200 OK）**

```json
{
  "success": true,
  "new_version": 4,
  "rotated_at": "2026-02-23T12:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_VAULT_SECRET_NOT_FOUND",
    "message": "secret not found at path: secret/data/k1s0/system/nonexistent",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（502 Bad Gateway）**

```json
{
  "error": {
    "code": "SYS_VAULT_UPSTREAM_ERROR",
    "message": "failed to rotate secret in Vault: permission denied",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/audit/logs

シークレットアクセスの監査ログをページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 50 | 1 ページあたりの件数 |
| `path` | string | No | - | シークレットパスフィルタ |
| `action` | string | No | - | アクションフィルタ（GET / ROTATE / LIST） |
| `service` | string | No | - | 要求元サービスフィルタ |

**レスポンス（200 OK）**

```json
{
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "path": "secret/data/k1s0/system/auth/database",
      "action": "GET",
      "requester_service": "auth-server",
      "requester_spiffe_id": "spiffe://k1s0/ns/k1s0-system/sa/auth-server",
      "version": 3,
      "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
      "created_at": "2026-02-23T10:00:00.000+00:00"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "path": "secret/data/k1s0/system/auth/database",
      "action": "ROTATE",
      "requester_service": "vault-admin",
      "requester_spiffe_id": "spiffe://k1s0/ns/k1s0-system/sa/vault-admin",
      "version": 4,
      "trace_id": "5bf92f3577b34da6a3ce929d0e0e4737",
      "created_at": "2026-02-23T12:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 1250,
    "page": 1,
    "page_size": 50,
    "has_next": true
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_VAULT_SECRET_NOT_FOUND` | 404 | 指定されたシークレットが Vault に見つからない |
| `SYS_VAULT_ACCESS_DENIED` | 403 | SPIFFE ID に基づくアクセス制御で拒否された |
| `SYS_VAULT_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー（不正なパス等） |
| `SYS_VAULT_UPSTREAM_ERROR` | 502 | Vault への接続・操作が失敗した |
| `SYS_VAULT_CACHE_ERROR` | 500 | キャッシュ操作の内部エラー |
| `SYS_VAULT_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.vault.v1;

service VaultService {
  rpc GetSecret(GetSecretRequest) returns (GetSecretResponse);
  rpc ListSecrets(ListSecretsRequest) returns (ListSecretsResponse);
  rpc RotateSecret(RotateSecretRequest) returns (RotateSecretResponse);
  rpc GetSecretMetadata(GetSecretMetadataRequest) returns (GetSecretMetadataResponse);
}

message GetSecretRequest {
  string path = 1;
  optional int32 version = 2;
}

message GetSecretResponse {
  map<string, string> data = 1;
  int32 version = 2;
  string created_at = 3;
}

message ListSecretsRequest {
  string path_prefix = 1;
}

message ListSecretsResponse {
  repeated string paths = 1;
}

message RotateSecretRequest {
  string path = 1;
  string reason = 2;
}

message RotateSecretResponse {
  bool success = 1;
  int32 new_version = 2;
  string rotated_at = 3;
}

message GetSecretMetadataRequest {
  string path = 1;
}

message GetSecretMetadataResponse {
  string path = 1;
  int32 current_version = 2;
  int32 oldest_version = 3;
  string created_at = 4;
  string updated_at = 5;
  string lease_duration = 6;
}
```

---

## キャッシュ設計

### シークレットキャッシュ

moka を使用した TTL ベースのインメモリキャッシュ。シークレット値をメモリ上に保持することで Vault への問い合わせ頻度を削減する。

| 設定項目 | 値 |
| --- | --- |
| キャッシュライブラリ | moka v0.12（非同期対応） |
| TTL 計算 | リース期限の 80%（例: リース 1h -> TTL 48min） |
| 最大エントリ数 | 10,000 |
| エビクションポリシー | TTL 期限切れ + LRU |
| キャッシュキー | `{path}:{version}` |

### キャッシュ無効化

以下のイベントでキャッシュエントリを無効化する。

```
1. ローテーション実行時 -> 対象パスのキャッシュをクリア
2. TTL 期限切れ -> moka が自動エビクション
3. Vault リース失効通知受信時 -> 対象パスのキャッシュをクリア
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
infrastructure（Vault Client・DB接続・Kafka Producer・キャッシュ・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Secret`, `SecretMetadata`, `SecretAccessLog` | エンティティ定義 |
| domain/repository | `SecretAccessLogRepository` | 監査ログリポジトリトレイト |
| domain/service | `VaultDomainService` | SPIFFE ベースのアクセス制御、キャッシュ管理ポリシー |
| usecase | `GetSecretUsecase`, `ListSecretsUsecase`, `RotateSecretUsecase`, `GetMetadataUsecase`, `LogAccessUsecase` | ユースケース |
| adapter/handler | REST ハンドラー, gRPC ハンドラー | プロトコル変換（axum / tonic） |
| adapter/gateway | `VaultClient` | Vault API クライアント（vaultrs 使用） |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `SecretAccessLogPostgresRepository` | PostgreSQL リポジトリ実装（監査ログ） |
| infrastructure/cache | `SecretCache` | moka キャッシュ（シークレット値 TTL キャッシュ） |
| infrastructure/messaging | `VaultEventPublisher`, `VaultKafkaProducer` | Kafka プロデューサー（ローテーション通知） |

### ドメインモデル

#### Secret

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `path` | String | シークレットパス（Vault KV v2 パス） |
| `version` | i32 | シークレットバージョン |
| `data` | Map\<String, String\> | シークレットデータ（key-value ペア） |
| `lease_duration` | Duration | リース期限 |
| `created_at` | DateTime\<Utc\> | 作成日時 |

#### SecretMetadata

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `path` | String | シークレットパス |
| `current_version` | i32 | 現在のバージョン |
| `oldest_version` | i32 | 最古のバージョン |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |
| `lease_duration` | Duration | リース期限 |

#### SecretAccessLog

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | 監査ログの一意識別子 |
| `path` | String | アクセスされたシークレットパス |
| `action` | String | アクション種別（GET / ROTATE / LIST） |
| `requester_service` | String | 要求元サービス名 |
| `requester_spiffe_id` | String | 要求元の SPIFFE ID |
| `version` | i32 | アクセスされたバージョン |
| `trace_id` | String | OpenTelemetry トレース ID |
| `created_at` | DateTime\<Utc\> | アクセス日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (vault_handler.rs)          │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  get_secret / get_metadata               │   │
                    │  │  list_secrets / rotate_secret            │   │
                    │  │  get_audit_logs                          │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (vault_grpc.rs)             │   │
                    │  │  VaultService impl                       │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ Gateway: VaultClient (vaultrs)           │   │
                    │  │  read_secret / list_secrets              │   │
                    │  │  write_secret / read_metadata            │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  GetSecretUsecase / ListSecretsUsecase /        │
                    │  RotateSecretUsecase / GetMetadataUsecase /     │
                    │  LogAccessUsecase                               │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────────┐         ┌──────────▼──────────────────┐   │
    │  domain/entity      │         │ domain/repository           │   │
    │  Secret,            │         │ SecretAccessLogRepository   │   │
    │  SecretMetadata,    │         │ (trait)                     │   │
    │  SecretAccessLog    │         └──────────┬─────────────────┘   │
    ├─────────────────────┤                    │                     │
    │  domain/service     │                    │                     │
    │  VaultDomainService │                    │                     │
    └─────────────────────┘                    │                     │
                                               │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ SecretAccessLog-       │  │
                    │  │ Producer     │  │ PostgresRepository     │  │
                    │  │ (rotation    │  │ (監査ログ)             │  │
                    │  │  notify)     │  │                        │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ moka Cache   │  │ Config                 │  │
                    │  │ (SecretCache)│  │ Loader                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐                              │
                    │  │ Database     │                              │
                    │  │ Config       │                              │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "vault-server"
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

vault:
  address: "http://vault.k1s0-system.svc.cluster.local:8200"
  auth_method: "approle"
  role_id: ""
  secret_id: ""
  mount_path: "secret"

cache:
  max_entries: 10000
  ttl_ratio: 0.8

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic: "k1s0.system.vault.rotated.v1"
```

---

## デプロイ

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。vault-server 固有の values は以下の通り。

```yaml
# values-vault.yaml（infra/helm/services/system/vault/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/vault-server
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
    - path: "secret/data/k1s0/system/vault-server/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/system/vault-server/approle"
      key: "secret_id"
      mountPath: "/vault/secrets/vault-secret-id"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/vault-server/database` |
| Vault AppRole Secret ID | `secret/data/k1s0/system/vault-server/approle` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-vault-server-実装設計.md](system-vault-server-実装設計.md) -- 実装設計の詳細
- [system-vault-server-デプロイ設計.md](system-vault-server-デプロイ設計.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

- [Vault設計.md](Vault設計.md) -- HashiCorp Vault 全体設計
- [サービス間認証設計.md](サービス間認証設計.md) -- SPIFFE/SPIRE によるサービス間認証
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [REST-API設計.md](REST-API設計.md) -- D-007 統一エラーレスポンス
- [メッセージング設計.md](メッセージング設計.md) -- Kafka イベント配信パターン
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [config設計.md](config設計.md) -- config.yaml スキーマ
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
