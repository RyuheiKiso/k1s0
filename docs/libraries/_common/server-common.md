# k1s0-server-common ライブラリ設計

Rust サーバー向けの内部共有ライブラリ。`SYS_{SERVICE}_{ERROR}` 形式のエラーコードと、統一エラーレスポンス型を提供する。

## 概要

`k1s0-server-common` は、system tier の Rust サーバーで重複しやすいエラー定義を共通化する。主な責務は以下。

- エラーコード型 (`ErrorCode`) の統一
- API エラーレスポンス型 (`ErrorResponse`) の統一
- サービス層エラー型 (`ServiceError`) の統一
- `axum` 利用時の `IntoResponse` 変換
- `utoipa` 利用時のスキーマ注釈対応

**配置先**: `regions/system/library/rust/server-common/`

## 言語サポート

`k1s0-server-common` は **Rust 専用** のライブラリ。  
Go / TypeScript / Dart には同名ライブラリは提供しない。

他言語では以下で代替する。

- エラーコード規約: `SYS_{SERVICE}_{ERROR}` を各言語実装で共通運用
- エラーレスポンス形状: 各サービスのハンドラー層で `{ "error": { ... } }` を統一
- OpenAPI/スキーマ連携: 各言語の標準ツールチェーン（Go: swag/chi, TS: zod/openapi, Dart: json_serializable など）で個別実装

## 公開 API

### 主要型

| 型 | 説明 |
| --- | --- |
| `ErrorCode` | `SYS_{SERVICE}_{ERROR}` 形式のコードラッパー |
| `ErrorDetail` | バリデーション等の詳細情報（`field` / `reason` / `message`） |
| `ErrorBody` | エラー本体（code/message/request_id/details） |
| `ErrorResponse` | `{ "error": ... }` の共通レスポンスラッパー。ビルダーメソッド `with_request_id(request_id)` でリクエスト ID を付与可能 |
| `ServiceError` | HTTP ステータスに対応したサービス層エラー |
| `PaginationResponse` | ページ情報（`total_count` / `page` / `page_size` / `has_next`） |
| `PaginatedResponse<T>` | `items` と `pagination` を持つ共通ページングレスポンス |

### well-known エラーコード

サービス別に補助関数を提供する。

- `error::auth::*`
- `error::config::*`
- `error::dlq::*`
- `error::tenant::*`
- `error::session::*`
- `error::api_registry::*`
- `error::event_store::*`
- `error::file::*`
- `error::scheduler::*`
- `error::notification::*`
- `error::featureflag::*`

### ErrorCode ファクトリメソッド

`ErrorCode` は以下のファクトリメソッドを提供する。

| メソッド | 生成コード |
| --- | --- |
| `ErrorCode::not_found(service)` | `SYS_{SERVICE}_NOT_FOUND` |
| `ErrorCode::validation(service)` | `SYS_{SERVICE}_VALIDATION_FAILED` |
| `ErrorCode::internal(service)` | `SYS_{SERVICE}_INTERNAL_ERROR` |
| `ErrorCode::unauthorized(service)` | `SYS_{SERVICE}_UNAUTHORIZED` |
| `ErrorCode::forbidden(service)` | `SYS_{SERVICE}_PERMISSION_DENIED` |
| `ErrorCode::conflict(service)` | `SYS_{SERVICE}_CONFLICT` |
| `ErrorCode::unprocessable(service)` | `SYS_{SERVICE}_BUSINESS_RULE_VIOLATION` |
| `ErrorCode::rate_exceeded(service)` | `SYS_{SERVICE}_RATE_EXCEEDED` |
| `ErrorCode::service_unavailable(service)` | `SYS_{SERVICE}_SERVICE_UNAVAILABLE` |
| `ErrorCode::biz_not_found(service)` | `BIZ_{SERVICE}_NOT_FOUND` |
| `ErrorCode::biz_validation(service)` | `BIZ_{SERVICE}_VALIDATION_FAILED` |
| `ErrorCode::svc_not_found(service)` | `SVC_{SERVICE}_NOT_FOUND` |
| `ErrorCode::svc_validation(service)` | `SVC_{SERVICE}_VALIDATION_FAILED` |

## Cargo 設定

```toml
[dependencies]
k1s0-server-common = { path = "../../system/library/rust/server-common", features = ["axum"] }
```

`features`:

- `axum`: `ServiceError` および `ErrorResponse` の `IntoResponse` 実装を有効化（HTTP レスポンス変換）
- `utoipa`: OpenAPI スキーマ生成向け derive を有効化

## 利用ガイド

1. ハンドラー層で `ServiceError` を返す
2. 必要に応じて `bad_request_with_details` で詳細情報を付与する
3. サービス固有コードは `ErrorCode::new("SYS_...")` または well-known 関数を利用する

```rust
use k1s0_server_common::ServiceError;

fn validate(name: &str) -> Result<(), ServiceError> {
    if name.is_empty() {
        return Err(ServiceError::bad_request("AUTH", "name is required"));
    }
    Ok(())
}
```

## Auth ユーティリティ

`auth` モジュールは、開発・テスト環境で認証を安全にバイパスするためのユーティリティ関数を提供する。クレートルートから再エクスポートされる。

| 関数 | シグネチャ | 説明 |
| --- | --- | --- |
| `allow_insecure_no_auth` | `(environment: &str) -> bool` | 環境変数 `ALLOW_INSECURE_NO_AUTH=true` が設定されており、かつ `environment` が `"dev"` または `"test"` の場合のみ `true` を返す。本番環境では常に `false`。 |
| `require_auth_state` | `<T>(service_name: &str, environment: &str, auth_state: Option<T>) -> Result<Option<T>>` | 認証設定（`auth_state`）の存在を検証する。`auth_state` が `None` の場合、`allow_insecure_no_auth` が `true` なら `Ok(None)` を返してバイパスし、`false` なら `ServiceError::unauthorized` を返す。 |

**環境変数**: `ALLOW_INSECURE_NO_AUTH` — `"true"` に設定すると dev/test 環境で認証バイパスを許可する。本番では設定しないこと。

## ミドルウェアスタック (`middleware` feature)

`middleware` feature を有効にすると、`K1s0Stack` ビルダーにより1行でミドルウェアスタック + 標準エンドポイントを構築できる。

### Cargo 設定

```toml
[dependencies]
k1s0-server-common = { path = "../../system/library/rust/server-common", features = ["middleware"] }
```

`middleware` feature は以下を自動有効化する: `axum`, `k1s0-telemetry` (axum-layer), `k1s0-correlation` (tower-layer), `k1s0-health`

### 使用例

```rust
use std::sync::Arc;
use k1s0_server_common::middleware::{K1s0Stack, Profile};
use k1s0_telemetry::metrics::Metrics;
use k1s0_health::CompositeHealthChecker;

let metrics = Arc::new(Metrics::new("config-server"));
let health_checker = Arc::new(CompositeHealthChecker::new());

let stack = K1s0Stack::new("config-server")
    .profile(Profile::from_env("prod"))
    .metrics(metrics.clone())
    .health_checker(health_checker);

let app = stack.wrap(handler::router(state));
```

### K1s0Stack API

| メソッド | 説明 |
| --- | --- |
| `K1s0Stack::new(service_name)` | ビルダーを生成（デフォルト: Dev, correlation有効, request_id有効） |
| `.profile(Profile)` | デプロイ環境を設定 |
| `.metrics(Arc<Metrics>)` | MetricsLayer 有効化 + `/metrics` エンドポイント追加 |
| `.health_checker(Arc<CompositeHealthChecker>)` | `/readyz` エンドポイント追加 |
| `.without_correlation()` | CorrelationLayer を無効化 |
| `.without_request_id()` | RequestIdLayer を無効化 |
| `.wrap(Router) -> Router` | ミドルウェアスタック適用 + 標準エンドポイント追加 |

### Profile

| 値 | 入力文字列 |
| --- | --- |
| `Profile::Prod` | `"prod"`, `"production"` |
| `Profile::Staging` | `"staging"`, `"stg"` |
| `Profile::Dev` | その他すべて |

### レイヤー適用順序（外→内）

1. **MetricsLayer** — 全リクエストの計測（`metrics` 設定時のみ）
2. **CorrelationLayer** — 相関ID注入・伝播（`x-correlation-id`, `x-trace-id`）
3. **RequestIdLayer** — リクエスト固有ID生成（`x-request-id`）

### 標準エンドポイント

| パス | 条件 | レスポンス |
| --- | --- | --- |
| `GET /healthz` | 常に追加 | `{"status":"ok"}` (200) |
| `GET /readyz` | `health_checker` 設定時 | HealthChecker結果 (200/503) |
| `GET /metrics` | `metrics` 設定時 | Prometheus text format |

### 個別レイヤー

`K1s0Stack` を使わず個別にレイヤーを適用することも可能。

- `k1s0_server_common::middleware::RequestIdLayer` — `x-request-id` ヘッダー付与
- `k1s0_correlation::layer::CorrelationLayer` — 相関ID注入（`k1s0-correlation` の `tower-layer` feature）
- `k1s0_telemetry::MetricsLayer` — HTTP メトリクス記録（`k1s0-telemetry` の `axum-layer` feature）

### Auth/RBAC について

認証・認可は K1s0Stack に含まない。ルート毎に異なる権限チェックが必要であり、`/healthz` や `/metrics` は認証不要のため、public/protected の分離は Router 層の責務とする。

## 関連ドキュメント

- [system-library 概要](./概要.md)
- [API 設計](../../architecture/api/API設計.md)
