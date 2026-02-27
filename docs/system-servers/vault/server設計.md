# system-vault-server 設計

system tier のシークレット管理サーバー設計を定義する。HashiCorp Vault 統合によるバージョン管理付き KV シークレットストアを提供し、Kafka 通知、SPIFFE 認証、監査ログに対応する。Rust での実装を定義する。

## 概要

system tier の Vault Server は以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| シークレット作成 | KV パス指定でシークレットを作成する（バージョン 1） |
| シークレット取得 | パス指定でシークレットを取得する（バージョン指定可能） |
| シークレット更新 | 既存シークレットを更新する（バージョンが自動インクリメントされる） |
| シークレット削除 | パス指定でシークレットを削除する |
| HashiCorp Vault 連携 | KV v2 / Dynamic Secrets 連携 |
| シークレットローテーション | 自動・手動によるシークレットローテーション |
| Kafka 通知 | `k1s0.system.vault.rotated.v1` トピックでローテーション通知を配信 |
| アクセス監査ログ | シークレットアクセスの監査ログを PostgreSQL に記録 |
| ヘルスモニタリング | ヘルスチェック・レディネスチェック・Prometheus メトリクス |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum 0.7 + tokio 1 |
| gRPC | tonic v0.12 + prost v0.13 |
| DB アクセス | sqlx v0.8（監査ログ用） |
| OTel | k1s0-telemetry |
| 設定管理 | serde_yaml |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |
| Vault クライアント | vaultrs |
| Kafka | rdkafka (rust-rdkafka) |
| キャッシュ | moka v0.12 |
| バリデーション | validator v0.18 |
| テスト | mockall 0.13, axum-test 16 |

### 配置パス

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/vault/` |

---

## 設計方針

[Vault設計.md](../../infrastructure/security/Vault設計.md) および [サービス間認証設計.md](../../auth/design/サービス間認証設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ストレージ | HashiCorp Vault 連携（KV v2） |
| バージョン管理 | シークレット更新時にバージョンが自動インクリメント（KV v2 互換） |
| アクセス制御 | SPIFFE ID ベースの認可 |
| キャッシュ | moka キャッシュ |
| Kafka | ローテーション通知 |
| 監査ログ | PostgreSQL に記録 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../api/gateway/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_VAULT_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/secrets` | シークレット作成 | `sys_operator` 以上 |
| GET | `/api/v1/secrets/:key` | シークレット取得 | SPIFFE ID ベースの認可 |
| PUT | `/api/v1/secrets/:key` | シークレット更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/secrets/:key` | シークレット削除 | `sys_admin` のみ |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/api/v1/secrets` | シークレット一覧 | `sys_auditor` 以上 |
| GET | `/api/v1/secrets/:key/metadata` | メタデータ取得 | `sys_auditor` 以上 |
| POST | `/api/v1/secrets/:key/rotate` | ローテーション | `sys_operator` 以上 |
| GET | `/api/v1/audit/logs` | 監査ログ | `sys_auditor` 以上 |
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

#### GET /api/v1/secrets/:key

指定パスのシークレットを取得する。バージョン未指定時は最新バージョンを返す。

**レスポンス（200 OK）**

```json
{
  "path": "app/db/password",
  "data": {
    "username": "db_admin",
    "password": "s3cret-v4lue"
  },
  "version": 1,
  "created_at": "2026-02-23T10:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": "secret not found: app/db/password"
}
```

#### PUT /api/v1/secrets/:key

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
  "updated_at": "2026-02-23T12:00:00.000+00:00"
}
```

#### DELETE /api/v1/secrets/:key

指定パスのシークレットを削除する。

**レスポンス（204 No Content）**

レスポンスボディなし。



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

`api/proto/k1s0/system/vault/v1/vault.proto` に定義。4 つの RPC（GetSecret / SetSecret / DeleteSecret / ListSecrets）を提供する。

```protobuf
syntax = "proto3";
package k1s0.system.vault.v1;

import "k1s0/system/common/v1/types.proto";

service VaultService {
  rpc GetSecret(GetSecretRequest) returns (GetSecretResponse);
  rpc SetSecret(SetSecretRequest) returns (SetSecretResponse);
  rpc DeleteSecret(DeleteSecretRequest) returns (DeleteSecretResponse);
  rpc ListSecrets(ListSecretsRequest) returns (ListSecretsResponse);
}

message GetSecretRequest {
  string path = 1;
  string version = 2;
}

message GetSecretResponse {
  map<string, string> data = 1;
  int64 version = 2;
  k1s0.system.common.v1.Timestamp created_at = 3;
}

message SetSecretRequest {
  string path = 1;
  map<string, string> data = 2;
}

message SetSecretResponse {
  int64 version = 1;
  k1s0.system.common.v1.Timestamp created_at = 2;
}

message DeleteSecretRequest {
  string path = 1;
  repeated int64 versions = 2;
}

message DeleteSecretResponse {
  bool success = 1;
}

message ListSecretsRequest {
  string path_prefix = 1;
}

message ListSecretsResponse {
  repeated string keys = 1;
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

```
domain（エンティティ・リポジトリインターフェース）
  ^
usecase（ビジネスロジック）
  ^
adapter（REST ハンドラー・gRPC ハンドラー）
  ^
infrastructure（Vault Client・DB・Kafka・キャッシュ）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Secret`, `SecretVersion`, `SecretValue`, `SecretAccessLog` | エンティティ定義 |
| domain/repository | `SecretStore`（trait）, `AccessLogRepository`（trait） | リポジトリトレイト |
| usecase | `GetSecretUseCase`, `SetSecretUseCase`, `DeleteSecretUseCase`, `ListSecretsUseCase` | ユースケース |
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

```yaml
app:
  name: k1s0-vault-server
  version: "0.1.0"
  environment: dev
server:
  host: "0.0.0.0"
  port: 8090
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

- [system-vault-server-実装設計.md](system-vault-server-実装設計.md) -- 実装設計の詳細
- [system-vault-server-デプロイ設計.md](system-vault-server-デプロイ設計.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

- [Vault設計.md](../../infrastructure/security/Vault設計.md) -- HashiCorp Vault 全体設計
- [サービス間認証設計.md](../../auth/design/サービス間認証設計.md) -- SPIFFE/SPIRE によるサービス間認証
- [API設計.md](../../api/gateway/API設計.md) -- REST API 設計ガイドライン
- [REST-API設計.md](../../api/protocols/REST-API設計.md) -- D-007 統一エラーレスポンス
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka イベント配信パターン
- [可観測性設計.md](../../observability/overview/可観測性設計.md) -- メトリクス・トレース設計
- [config設計.md](../../cli/config/config設計.md) -- config.yaml スキーマ
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート仕様
- [tier-architecture.md](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
