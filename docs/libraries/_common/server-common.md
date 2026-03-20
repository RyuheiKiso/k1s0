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

`k1s0-server-common` は以下の言語で実装されている。

| 言語 | パッケージ名 | 配置パス | 機能範囲 |
|------|-------------|---------|---------|
| Rust | `k1s0-server-common` | `regions/system/library/rust/server-common/` | 全機能（axum 統合・ミドルウェア・well-known コード） |
| Go | `server-common` | `regions/system/library/go/server-common/` | HTTP+gRPC 統合起動基盤 |
| TypeScript | `@k1s0/server-common` | `regions/system/library/typescript/server-common/` | エラーコード・レスポンス型・well-known コード（クライアント向け） |
| Dart | `k1s0_server_common` | `regions/system/library/dart/server_common/` | エラーコード・レスポンス型・well-known コード（クライアント向け） |

TypeScript / Dart 版はクライアント向けの共通型定義に特化し、axum 統合やミドルウェアスタックは含まない。エラーコード規約（`SYS_{SERVICE}_{ERROR}`）と well-known コード定数は全言語で同一。

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
| `ObservabilityConfig` | 可観測性設定の親構造体（`config` モジュール）。`LogConfig` / `TraceConfig` / `MetricsConfig` を保持する |
| `LogConfig` | ログレベル・フォーマット設定（`config` モジュール） |
| `TraceConfig` | 分散トレース設定（`config` モジュール） |
| `MetricsConfig` | Prometheus メトリクス設定（`config` モジュール） |

### config モジュール（共通設定構造体）

技術監査対応で追加された `config` モジュールは、全サーバーで重複していた可観測性設定構造体を一箇所に集約する。`ObservabilityConfig` / `LogConfig` / `TraceConfig` / `MetricsConfig` を提供し、`serde(default)` によるデフォルト値の自動適用に対応する。詳細は [Rust共通実装リファレンス](../../servers/_common/Rust共通実装.md#共通observabilityconfig) を参照。

```rust
use k1s0_server_common::config::ObservabilityConfig;
```

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

### 定数

| 定数 | 型 | 説明 |
| --- | --- | --- |
| `DEFAULT_OTEL_ENDPOINT` | `&str` | デフォルトの OpenTelemetry コレクターエンドポイント（`http://otel-collector.observability:4317`）。全サーバーの設定デフォルト値として使用する。エンドポイント変更時はここだけ修正すればよい。 |

## Cargo 設定

```toml
[dependencies]
k1s0-server-common = { path = "../../system/library/rust/server-common", features = ["axum"] }
```

`features`:

- `axum`: `ServiceError` および `ErrorResponse` の `IntoResponse` 実装を有効化（HTTP レスポンス変換）
- `utoipa`: OpenAPI スキーマ生成向け derive を有効化
- `dev-auth-bypass`: 認証バイパス（`ALLOW_INSECURE_NO_AUTH`）のコードパスを有効化する。リリースビルドでこのフィーチャーを無効にすると、バイパスロジックがバイナリから完全に除去される。デバッグビルドでは `debug_assertions` により自動的に有効。

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
| `allow_insecure_no_auth` | `(environment: &str) -> bool` | **デバッグビルドまたは `dev-auth-bypass` フィーチャー有効時のみ動作**: 環境変数 `ALLOW_INSECURE_NO_AUTH=true` が設定されており、かつ `environment` が `"dev"` または `"test"` の場合のみ `true` を返す。**リリースビルドかつフィーチャー無効時は `#[cfg(not(any(debug_assertions, feature = "dev-auth-bypass")))]` により常に `false` を返し、バイパスロジックはバイナリから完全に除去される。** Docker ローカル開発時は `CARGO_FEATURES="k1s0-server-common/dev-auth-bypass"` ビルド引数で有効化する。 |
| `require_auth_state` | `<T>(service_name: &str, environment: &str, auth_state: Option<T>) -> Result<Option<T>>` | 認証設定（`auth_state`）の存在を検証する。`auth_state` が `None` の場合、`allow_insecure_no_auth` が `true` なら `Ok(None)` を返してバイパスし、`false` なら `ServiceError::unauthorized` を返す。 |

**環境変数**: `ALLOW_INSECURE_NO_AUTH` — `"true"` に設定すると dev/test 環境で認証バイパスを許可する。ただし以下の条件を両方満たす必要がある: (1) デバッグビルドまたは `dev-auth-bypass` フィーチャーが有効、(2) `environment` が `"dev"` または `"test"`。本番リリースビルドではフィーチャー未指定のためコンパイル時に除去済み。

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

### K1s0App（上位ビルダー）

`K1s0App` は Config → Telemetry → Metrics → HealthCheck → K1s0Stack の初期化を一括で行う上位ビルダー。新規サーバー作成時のボイラープレートを 50〜80行から 5〜15行に削減する。

```rust
use k1s0_server_common::middleware::{K1s0App, K1s0AppReady};

let app = K1s0App::new(cfg.to_telemetry_config())
    .add_health_check(Box::new(db_check))
    .build()
    .await?;
let router = app.wrap(handler::router(state));
```

#### K1s0App API

| メソッド | 説明 |
| --- | --- |
| `K1s0App::new(TelemetryConfig)` | ビルダーを生成 |
| `.profile(Profile)` | デプロイ環境を明示指定（未指定時は `TelemetryConfig.environment` から自動判定） |
| `.add_health_check(Box<dyn HealthCheck>)` | HealthCheck を追加 |
| `.without_correlation()` | CorrelationLayer を無効化 |
| `.without_request_id()` | RequestIdLayer を無効化 |
| `.build() -> Result<K1s0AppReady>` | Telemetry初期化 → Metrics生成 → HealthChecker構築 → Ready返却 |

#### K1s0AppReady API

| メソッド | 説明 |
| --- | --- |
| `.service_name() -> &str` | サービス名を返す |
| `.profile() -> &Profile` | 適用されたProfileを返す |
| `.metrics() -> Arc<Metrics>` | Metricsインスタンスを返す |
| `.health_checker() -> Arc<CompositeHealthChecker>` | HealthCheckerを返す |
| `.wrap(Router) -> Router` | K1s0Stackを内部構築してRouterに適用 |

#### K1s0App vs K1s0Stack

- **K1s0App**: Telemetry・Metrics・HealthCheck の生成まで含む。新規サーバーのエントリポイント向け
- **K1s0Stack**: 既に生成済みの `Arc<Metrics>` 等を受け取る低レベルビルダー。カスタム初期化が必要な場合に使用

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
