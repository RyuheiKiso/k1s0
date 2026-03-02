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
| `ErrorDetail` | バリデーション等の詳細情報（field/reason/value） |
| `ErrorBody` | エラー本体（code/message/request_id/details） |
| `ErrorResponse` | `{ "error": ... }` の共通レスポンスラッパー |
| `ServiceError` | HTTP ステータスに対応したサービス層エラー |

### well-known エラーコード

サービス別に補助関数を提供する。

- `error::auth::*`
- `error::config::*`
- `error::dlq::*`
- `error::tenant::*`
- `error::session::*`
- `error::api_registry::*`

## Cargo 設定

```toml
[dependencies]
k1s0-server-common = { path = "../../system/library/rust/server-common", features = ["axum"] }
```

`features`:

- `axum`: `ServiceError` / `ErrorResponse` の HTTP レスポンス変換を有効化
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

## 関連ドキュメント

- [system-library 概要](./概要.md)
- [API 設計](../../architecture/api/API設計.md)
