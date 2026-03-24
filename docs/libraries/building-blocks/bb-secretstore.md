# bb-secretstore ライブラリ設計

## 概要

シークレット（パスワード・API キー・証明書等）の安全な取得を抽象化する SecretStore Building Block クレート。キー単位の取得（`get_secret`）と複数キーの一括取得（`bulk_get`）を統一インターフェースで提供する。本番実装として HashiCorp Vault バックエンド（`k1s0-vault-client` をラップ）、開発・テスト用としてインメモリ実装を備える。

**配置先**: `regions/system/library/rust/bb-secretstore/`

**パッケージ名**: `k1s0-bb-secretstore`

## 設計思想

アプリケーションコードからシークレット管理バックエンドの詳細（Vault の API・認証方式等）を隠蔽し、ローカル開発・CI・本番環境で同一コードを動作させる。`bb-core` の `Component` トレイトを拡張することで、シークレットストアも他の BB コンポーネントと同じライフサイクル管理に統合される。`bulk_get` では `NotFound` エラーを無視してスキップする設計とし、複数シークレット取得時の部分的欠落を呼び出し側に委ねる。

## トレイト定義

### データ型

| 型 | 説明 |
|----|------|
| `SecretValue` | シークレットストアのエントリ（`key: String`・`value: String`・`metadata: HashMap<String, String>`） |

### SecretStore トレイト

シークレット管理の抽象インターフェース。`k1s0_bb_core::Component` を拡張する。

```rust
#[async_trait]
pub trait SecretStore: k1s0_bb_core::Component {
    /// キーに対応するシークレット値を取得する。
    async fn get_secret(&self, key: &str) -> Result<SecretValue, SecretStoreError>;

    /// 複数キーのシークレット値を一括取得する。
    /// 存在しないキーは結果から除外される（エラーにならない）。
    async fn bulk_get(
        &self,
        keys: &[&str],
    ) -> Result<HashMap<String, SecretValue>, SecretStoreError>;
}
```

## 実装バリエーション

### Vault SecretStore（`VaultSecretStore`、`vault` feature 必須）

`k1s0-vault-client` の `VaultClient` トレイトをラップした HashiCorp Vault バックエンドの本番向け実装。

| 項目 | 内容 |
|------|------|
| `component_type` | `"secretstore"` |
| metadata `backend` | `"vault"` |
| `get_secret()` の挙動 | `VaultClient::get_secret()` を呼び出し、`Secret.data` を `SecretValue` に変換する。データが1エントリの場合はその値を、複数の場合は JSON シリアライズした文字列を `value` に格納する。`metadata` にはシークレットのバージョン（`version`）を含む |
| `bulk_get()` の挙動 | 各キーに対して `get_secret()` を順次呼び出す。`NotFound` はスキップし、その他のエラーは即座に返す |

コンストラクタ:
- `VaultSecretStore::new(name, client)` — `Arc<dyn VaultClient>` を注入

### インメモリ SecretStore（`InMemorySecretStore`）

`HashMap` をバックエンドとするテスト・開発用実装。

| 項目 | 内容 |
|------|------|
| `component_type` | `"secretstore"` |
| metadata `backend` | `"memory"` |
| `get_secret()` の挙動 | キーが存在しない場合は `SecretStoreError::NotFound` を返す |
| `bulk_get()` の挙動 | 存在するキーのみを結果に含める（エラーにならない） |
| `close()` の挙動 | ストアの全エントリをクリアする |

テスト用ヘルパーメソッド:
- `put_secret(key, value)` — シークレットを追加する
- `put_secret_with_metadata(key, value, metadata)` — メタデータ付きシークレットを追加する

## 使用例

```rust
use std::collections::HashMap;
use k1s0_bb_secretstore::{InMemorySecretStore, SecretStore};

// インメモリ実装でシークレットを管理する（テスト・ローカル開発用）
let store = InMemorySecretStore::new("app-secrets");
store.init().await?;

// テスト用シークレットを登録する
store.put_secret("db/password", "s3cr3t_password").await;
store.put_secret("api/key", "abc123xyz").await;

// 単一シークレットを取得する
let secret = store.get_secret("db/password").await?;
println!("DB パスワード: {}", secret.value);

// 複数シークレットを一括取得する（存在しないキーはスキップされる）
let secrets = store
    .bulk_get(&["db/password", "api/key", "nonexistent/key"])
    .await?;

// 取得できたシークレットのみ処理する
assert_eq!(secrets.len(), 2);
let db_pass = secrets.get("db/password").unwrap();
println!("DB パスワード: {}", db_pass.value);

// 本番環境では VaultSecretStore に差し替える（vault feature 使用時）
// use k1s0_bb_secretstore::VaultSecretStore;
// let vault = VaultSecretStore::new("vault", Arc::new(vault_client));
// vault.init().await?;
// let secret = vault.get_secret("secret/db/password").await?;
```

## エラーハンドリング

`SecretStoreError` は `thiserror` で定義された enum。

| バリアント | 説明 |
|-----------|------|
| `NotFound(String)` | 指定したキーのシークレットが存在しない |
| `PermissionDenied(String)` | シークレットへのアクセス権限がない |
| `Connection(String)` | Vault サーバー等への接続エラー |
| `Authentication(String)` | Vault への認証エラー |
| `Component(ComponentError)` | `bb-core` のコンポーネントエラー（`#[from]` 自動変換） |

`bulk_get` では `NotFound` は無視してスキップする動作のため、呼び出し側は返却された `HashMap` のキーを確認して欠落の有無を判断すること。

## 依存関係

| クレート | 用途 |
|---------|------|
| `k1s0-bb-core` | `Component` トレイト・`ComponentError`・`ComponentStatus` |
| `async-trait` | 非同期トレイト定義 |
| `serde` | シリアライズ（将来拡張用） |
| `serde_json` | Vault の複数フィールドシークレットの JSON シリアライズ |
| `thiserror` | エラー型の派生 |
| `tokio` | 非同期ランタイム・`RwLock` |
| `tracing` | 構造化ログ出力 |
| `k1s0-vault-client` | HashiCorp Vault クライアント（`vault` feature 有効時のみ） |
| `mockall` | モック生成（`mock` feature 有効時のみ） |

### Feature フラグ

| Feature | 説明 |
|---------|------|
| `vault` | `k1s0-vault-client` による `VaultSecretStore` を有効化 |
| `mock` | `mockall` によるモック実装を有効化（テスト用） |

**依存追加**: `k1s0-bb-secretstore = { path = "../../system/library/rust/bb-secretstore" }`

Vault 実装を使用する場合: `k1s0-bb-secretstore = { path = "../../system/library/rust/bb-secretstore", features = ["vault"] }`

## 関連ドキュメント

- [Building Blocks 抽象化](../_common/building-blocks.md) — BB アーキテクチャ全体設計
- [bb-core 設計](bb-core.md) — BB 基盤クレート
- [SecretStore 設計](../../architecture/auth-security/secret-store.md) — SecretStore Building Block 詳細
