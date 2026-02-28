# system-ratelimit-server 設計

Redis トークンバケットによるレート制限判定サーバー。Kong 連携・内部サービス間保護を提供。

> **ガイド**: 実装例・設定ファイル・依存関係図は [server.guide.md](./server.guide.md) を参照。

## 概要

system tier のレートリミットサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| レート制限判定 | サービス・ユーザー・エンドポイントをキーとしたレート制限チェック |
| リミット設定管理 | リミットルールの作成・更新・削除・一覧取得 |
| リミットリセット | 緊急時の特定キーのリミットカウンターリセット |
| 使用量照会 | 現在の使用量・残余リクエスト数・リセット時刻の取得 |
| Lua スクリプト | Redis Lua スクリプトによるアトミックなトークンバケット実装 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Redis | redis v0.27（Lua スクリプト対応） |

### 配置パス

配置: `regions/system/server/rust/ratelimit/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) のレート制限方針に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| アルゴリズム | トークンバケット（Redis Lua スクリプトでアトミック実装） |
| キー設計 | `ratelimit:{scope}:{identifier}:{window}` 形式（scope: service/user/endpoint） |
| ウィンドウ | 固定ウィンドウ（60 秒）と設定可能ウィンドウをサポート |
| ルール永続化 | PostgreSQL の `ratelimit` スキーマ。Redis は判定状態のみ保持 |
| Redis 障害時 | フェイルオープン（障害時はリミットを通過させる）。設定で変更可能 |
| Kong 連携 | Kong プラグインから gRPC で `CheckRateLimit` を呼び出す |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_RATE_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/ratelimit/check` | レート制限チェック | 不要（内部サービス用） |
| POST | `/api/v1/ratelimit/rules` | ルール作成 | `sys_operator` 以上 |
| GET | `/api/v1/ratelimit/rules/:id` | ルール取得 | `sys_auditor` 以上 |
| GET | `/api/v1/ratelimit/usage` | 使用量照会 | `sys_auditor` 以上 |
| GET | `/api/v1/ratelimit/rules` | ルール一覧取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/ratelimit/rules/:id` | ルール更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/ratelimit/rules/:id` | ルール削除 | `sys_admin` のみ |
| POST | `/api/v1/ratelimit/reset` | カウンターリセット | `sys_operator` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/ratelimit/check

指定されたスコープ・識別子に対してレート制限チェックを行う。Redis のトークンバケットからトークンを消費し、許可/拒否を返す。内部サービス用のエンドポイントであり、認証は不要。

#### GET /api/v1/ratelimit/usage

指定されたスコープ・識別子の現在の使用量を照会する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `scope` | string | Yes | - | スコープ（service/user/endpoint） |
| `identifier` | string | Yes | - | 識別子（サービス名/ユーザー ID/エンドポイントパス） |

#### GET /api/v1/ratelimit/rules

レートリミットルール一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `scope` | string | No | - | スコープでフィルタ |
| `enabled_only` | bool | No | false | 有効なルールのみ取得 |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

#### POST /api/v1/ratelimit/rules

新しいレートリミットルールを作成する。

#### PUT /api/v1/ratelimit/rules/:id

既存のレートリミットルールを更新する。

#### DELETE /api/v1/ratelimit/rules/:id

レートリミットルールを削除する。

#### POST /api/v1/ratelimit/reset

指定されたスコープ・識別子のレートリミットカウンターをリセットする。緊急時に使用する。

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_RATE_NOT_FOUND` | 404 | 指定されたルールまたは状態が見つからない |
| `SYS_RATE_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_RATE_REDIS_ERROR` | 503 | Redis 接続エラー（フェイルオープン設定時は 200 で通過） |
| `SYS_RATE_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.ratelimit.v1;

service RateLimitService {
  rpc CheckRateLimit(CheckRateLimitRequest) returns (CheckRateLimitResponse);
  rpc GetUsage(GetUsageRequest) returns (GetUsageResponse);
  rpc ResetLimit(ResetLimitRequest) returns (ResetLimitResponse);
}

message CheckRateLimitRequest {
  string scope = 1;
  string identifier = 2;
  optional string window = 3;
}

message CheckRateLimitResponse {
  bool allowed = 1;
  uint32 remaining = 2;
  uint64 reset_at = 3;
  uint32 limit = 4;
  string reason = 5;
}

message GetUsageRequest {
  string scope = 1;
  string identifier = 2;
}

message GetUsageResponse {
  uint32 used = 1;
  uint32 limit = 2;
  uint32 remaining = 3;
  uint64 reset_at = 4;
}

message ResetLimitRequest {
  string scope = 1;
  string identifier = 2;
}

message ResetLimitResponse {
  bool success = 1;
}
```

---

## トークンバケット実装

### キー設計

| スコープ | キーフォーマット | 例 |
| --- | --- | --- |
| service | `ratelimit:service:{service_name}:{window}` | `ratelimit:service:order-service:60` |
| user | `ratelimit:user:{user_id}:{window}` | `ratelimit:user:user-001:60` |
| endpoint | `ratelimit:endpoint:{path}:{window}` | `ratelimit:endpoint:/api/v1/orders:60` |

### フェイルオープン動作

Redis が利用不能な場合のフォールバック動作。

| 設定値 | 動作 |
| --- | --- |
| `fail_open: true`（デフォルト） | Redis 障害時はリクエストを許可（`allowed: true`, `reason: "redis unavailable, fail-open"` ） |
| `fail_open: false` | Redis 障害時はリクエストを拒否（`allowed: false`, `reason: "redis unavailable, fail-closed"`） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `RateLimitRule`, `RateLimitStatus`, `RateLimitCheck` | エンティティ定義 |
| domain/repository | `RateLimitRuleRepository`（PostgreSQL）, `RateLimitStateRepository`（Redis） | リポジトリトレイト |
| domain/service | `RateLimitDomainService` | トークンバケット判定ロジック |
| usecase | `CheckRateLimitUsecase`, `GetUsageUsecase`, `ResetLimitUsecase`, `CreateRuleUsecase`, `UpdateRuleUsecase`, `DeleteRuleUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `RateLimitRulePostgresRepository` | PostgreSQL リポジトリ実装（ルール永続化） |
| infrastructure/cache | `RateLimitRedisRepository` + Lua スクリプト | Redis リポジトリ実装（状態管理） |

### ドメインモデル

#### RateLimitRule

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ルールの一意識別子 |
| `scope` | String | スコープ（service / user / endpoint） |
| `identifier_pattern` | String | 識別子パターン（`*` でワイルドカード、特定値で個別指定） |
| `limit` | u32 | ウィンドウあたりの最大リクエスト数 |
| `window_seconds` | u32 | ウィンドウサイズ（秒） |
| `enabled` | bool | ルールの有効/無効 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### RateLimitStatus

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `key` | String | Redis キー（`ratelimit:{scope}:{identifier}:{window}`） |
| `used` | u32 | 現在のウィンドウでの使用数 |
| `limit` | u32 | ウィンドウあたりの最大リクエスト数 |
| `remaining` | u32 | 残余リクエスト数 |
| `reset_at` | DateTime\<Utc\> | ウィンドウリセット時刻 |

#### RateLimitCheck

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `scope` | String | チェック対象のスコープ |
| `identifier` | String | チェック対象の識別子 |
| `allowed` | bool | 許可/拒否の判定結果 |
| `remaining` | u32 | 残余リクエスト数 |
| `reset_at` | DateTime\<Utc\> | ウィンドウリセット時刻 |
| `rule_id` | UUID | 適用されたルールの ID |

### ルールマッチング

リクエストに対するルール検索の優先順位: (1) scope + identifier 完全一致 → (2) scope + ワイルドカード(`*`) → (3) デフォルトルール。

---

## Kong 連携

### レスポンスヘッダー

Kong プラグインは ratelimit-server のレスポンスに基づいて以下のヘッダーを付与する。

| ヘッダー | 値 | 説明 |
| --- | --- | --- |
| `X-RateLimit-Limit` | `100` | ウィンドウあたりの最大リクエスト数 |
| `X-RateLimit-Remaining` | `95` | 残余リクエスト数 |
| `X-RateLimit-Reset` | `1740052260` | リセット時刻（Unix timestamp） |
| `Retry-After` | `45` | 429 レスポンス時のみ、リトライまでの秒数 |

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/ratelimit/database` |
| Redis パスワード | `secret/data/k1s0/system/ratelimit/redis` |

---

## 詳細設計ドキュメント

- [system-ratelimit-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-ratelimit-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) -- Kong API ゲートウェイ設計
