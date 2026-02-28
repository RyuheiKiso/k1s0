# system-policy-server 設計

OPA 連携の動的ポリシー評価サーバー。Rego ポリシー管理・バンドル管理・評価キャッシュを提供。

> **ガイド**: 実装例・設定ファイル・依存関係図は [server.guide.md](./server.guide.md) を参照。

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

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) および [RBAC設計.md](../../architecture/auth/RBAC設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ポリシーエンジン | OPA（Open Policy Agent）。opa-client クレート経由で OPA HTTP API を呼び出す |
| ポリシー言語 | Rego。ポリシー本文は PostgreSQL の `policy.policies` テーブルで管理 |
| 評価フロー | REST/gRPC リクエスト → Rust サーバー → OPA HTTP API（/v1/data/{package}） → 評価結果返却 |
| キャッシュ | moka で評価結果を TTL 30 秒キャッシュ。Kafka 通知受信時にキャッシュ無効化 |
| DB スキーマ | PostgreSQL の `policy` スキーマ（policies, policy_bundles テーブル） |
| Kafka | ポリシー変更時に `k1s0.system.policy.updated.v1` トピックへ変更通知を送信 |
| ポート | ホスト側 8096（内部 8080）、gRPC 9090 |

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

登録済みポリシーの一覧をページネーション付きで取得する。`bundle_id` クエリパラメータでバンドル別にフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `bundle_id` | string | No | - | バンドル ID でフィルタ |
| `enabled_only` | bool | No | false | 有効なポリシーのみ取得 |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

#### GET /api/v1/policies/:id

ID 指定でポリシーの詳細を取得する。

#### POST /api/v1/policies

新しい Rego ポリシーを作成する。作成時に OPA への同期も行い、Kafka 変更通知を送信する。

#### PUT /api/v1/policies/:id

既存のポリシーを更新する。更新時にバージョンを自動インクリメントし、OPA への同期、Kafka 変更通知を行う。キャッシュは即座に無効化される。

#### DELETE /api/v1/policies/:id

ポリシーを削除する。削除時に OPA からもポリシーを削除し、Kafka 変更通知を送信する。

#### POST /api/v1/policies/:id/evaluate

指定ポリシーに対して入力データを評価し、allow/deny を返す。評価結果は moka キャッシュに TTL 30 秒で保存される。

#### GET /api/v1/bundles

登録済みバンドルの一覧を取得する。

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

```protobuf
syntax = "proto3";
package k1s0.system.policy.v1;

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
  string created_at = 9;
  string updated_at = 10;
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

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Policy`, `PolicyBundle`, `PolicyEvaluation` | エンティティ定義 |
| domain/repository | `PolicyRepository`, `PolicyBundleRepository` | リポジトリトレイト |
| domain/service | `PolicyDomainService` | Rego 構文バリデーション・評価結果キャッシュキー生成 |
| usecase | `GetPolicyUsecase`, `ListPoliciesUsecase`, `CreatePolicyUsecase`, `UpdatePolicyUsecase`, `DeletePolicyUsecase`, `EvaluatePolicyUsecase`, `ListBundlesUsecase` | ユースケース |
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
| `description` | String | バンドルの説明 |
| `policy_count` | u32 | 所属ポリシー数 |
| `enabled` | bool | バンドルの有効/無効 |
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
