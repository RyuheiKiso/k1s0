# bb-statestore ライブラリ設計

## 概要

キーバリュー形式の状態管理（StateStore）を抽象化する Building Block クレート。キーの取得・設定・削除と複数キーの一括操作、および楽観的ロック（ETag ベース）を統一インターフェースで提供する。本番実装として Redis バックエンド（`k1s0-cache` の `CacheClient` をラップ）、開発・テスト用としてインメモリ実装を備える。

**配置先**: `regions/system/library/rust/bb-statestore/`

**パッケージ名**: `k1s0-bb-statestore`

## 設計思想

マイクロサービスのセッション状態・キャッシュ・分散ロックなどの状態管理において、バックエンドストア（Redis・インメモリ等）の詳細をアプリケーションコードから隠蔽する。ETag による楽観的ロックを標準インターフェースに組み込み、競合検出を一貫した方法で扱えるようにする。`bb-core` の `Component` トレイトを拡張することで、ステートストアも他の BB コンポーネントと同じライフサイクル管理（init / close / status）に統合される。Redis 実装では ETag を `{key}:__etag` という専用キーに保存することで、既存の Redis データ構造に干渉しない設計とする。

## トレイト定義

### データ型

| 型 | 説明 |
|----|------|
| `StateEntry` | ステートストアのエントリ（`key: String`・`value: Vec<u8>`・`etag: String`） |

### StateStore トレイト

キーバリューストアの抽象インターフェース。`k1s0_bb_core::Component` を拡張する。

```rust
#[async_trait]
pub trait StateStore: k1s0_bb_core::Component {
    /// キーに対応する値を取得する。キーが存在しない場合は None を返す。
    async fn get(&self, key: &str) -> Result<Option<StateEntry>, StateStoreError>;

    /// キーに値を設定し、新しい ETag を返す。
    /// etag を指定した場合、現在の ETag と一致しないと ETagMismatch エラーを返す。
    async fn set(
        &self,
        key: &str,
        value: &[u8],
        etag: Option<&str>,
    ) -> Result<String, StateStoreError>;

    /// キーを削除する。
    /// etag を指定した場合、現在の ETag と一致しないと ETagMismatch エラーを返す。
    async fn delete(&self, key: &str, etag: Option<&str>) -> Result<(), StateStoreError>;

    /// 複数キーを一括取得する。存在するキーのエントリのみを返す。
    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<StateEntry>, StateStoreError>;

    /// 複数エントリを一括設定し、各キーの新しい ETag のリストを返す。
    async fn bulk_set(&self, entries: &[(&str, &[u8])]) -> Result<Vec<String>, StateStoreError>;
}
```

## 実装バリエーション

### Redis StateStore（`RedisStateStore`、`redis` feature 必須）

`k1s0-cache` の `CacheClient` トレイトをラップした Redis バックエンドの本番向け実装。

| 項目 | 内容 |
|------|------|
| `component_type` | `"statestore"` |
| metadata `backend` | `"redis"` |
| ETag 保存方式 | データキー `{key}` に加え、ETag を `{key}:__etag` キーに別途保存する |
| `get()` の挙動 | データと ETag を別々に Redis から取得して `StateEntry` に組み立てる |
| `set()` の挙動 | ETag 指定時は現在の ETag を確認し不一致なら `ETagMismatch` を返す。成功時は UUID で新 ETag を生成して保存する |
| `delete()` の挙動 | ETag 指定時は現在の ETag を確認し不一致なら `ETagMismatch` を返す。データキーと ETag キーの両方を削除する |
| `bulk_set()` の挙動 | ETag 指定なしで各エントリを順次 `set()` する |

コンストラクタ:
- `RedisStateStore::new(name, client)` — `Arc<dyn CacheClient>` を注入

### インメモリ StateStore（`InMemoryStateStore`）

`HashMap` をバックエンドとするテスト・開発用実装。ETag 付き内部エントリ（`Entry { value, etag }`）で管理する。

| 項目 | 内容 |
|------|------|
| `component_type` | `"statestore"` |
| metadata `backend` | `"memory"` |
| `get()` の挙動 | キーが存在しない場合は `None` を返す |
| `set()` の挙動 | ETag 指定時は既存エントリの ETag と比較し不一致なら `ETagMismatch` を返す。成功時は UUID で新 ETag を生成して保存する |
| `delete()` の挙動 | ETag 指定時は既存エントリの ETag と比較し不一致なら `ETagMismatch` を返す。存在しないキーは無視する |
| `bulk_set()` の挙動 | ETag 指定なしで各エントリを一括挿入する（単一 write ロックで実行） |
| `close()` の挙動 | ストアの全エントリをクリアする |

## 使用例

```rust
use k1s0_bb_statestore::{InMemoryStateStore, StateStore};

// インメモリ実装で状態を管理する（テスト・ローカル開発用）
let store = InMemoryStateStore::new("session-store");
store.init().await?;

// 値を設定する（ETag が返される）
let etag = store.set("session:user123", b"active", None).await?;
println!("ETag: {}", etag);

// 値を取得する
if let Some(entry) = store.get("session:user123").await? {
    println!("値: {:?}", entry.value);
    println!("ETag: {}", entry.etag);
}

// 楽観的ロックで更新する（ETag が一致する場合のみ更新）
let new_etag = store
    .set("session:user123", b"inactive", Some(&etag))
    .await?;

// ETag が不一致の場合は ETagMismatch エラーになる
let result = store
    .set("session:user123", b"other", Some("stale-etag"))
    .await;
assert!(result.is_err());

// 複数キーを一括取得する（存在するキーのみ返される）
let entries = store
    .bulk_get(&["session:user123", "session:user456", "nonexistent"])
    .await?;
println!("取得件数: {}", entries.len()); // 存在するキーのみ

// 複数エントリを一括設定する
let etags = store
    .bulk_set(&[
        ("config:timeout", b"30"),
        ("config:retry", b"3"),
    ])
    .await?;
println!("設定した ETag 数: {}", etags.len());

// 本番環境では RedisStateStore に差し替える（redis feature 使用時）
// use k1s0_bb_statestore::RedisStateStore;
// let redis = RedisStateStore::new("redis-store", Arc::new(cache_client));
// redis.init().await?;
// let etag = redis.set("session:user123", b"active", None).await?;
```

## エラーハンドリング

`StateStoreError` は `thiserror` で定義された enum。

| バリアント | 説明 |
|-----------|------|
| `NotFound(String)` | 指定したキーが存在しない（現在は未使用。`get()` は `None` を返す設計） |
| `ETagMismatch { expected, actual }` | 楽観的ロックの ETag 不一致。`expected`（指定値）と `actual`（現在値）を含む |
| `Connection(String)` | Redis 等への接続エラー |
| `Serialization(String)` | 値のシリアライズ・デシリアライズエラー（Redis では UTF-8 変換失敗等） |
| `Component(ComponentError)` | `bb-core` のコンポーネントエラー（`#[from]` 自動変換） |

楽観的ロックの利用パターン:
1. `get()` で現在の `StateEntry.etag` を取得する
2. `set()` / `delete()` に取得した `etag` を渡す
3. 競合が発生した場合（他プロセスが先に更新）は `ETagMismatch` が返るので、`get()` から再試行する

## 依存関係

| クレート | 用途 |
|---------|------|
| `k1s0-bb-core` | `Component` トレイト・`ComponentError`・`ComponentStatus` |
| `async-trait` | 非同期トレイト定義 |
| `serde` | シリアライズ（将来拡張用） |
| `thiserror` | エラー型の派生 |
| `tokio` | 非同期ランタイム・`RwLock` |
| `tracing` | 構造化ログ出力 |
| `uuid` | ETag（UUID v4）の生成 |
| `k1s0-cache` | Redis `CacheClient` トレイト（`redis` feature 有効時のみ） |
| `mockall` | モック生成（`mock` feature 有効時のみ） |

### Feature フラグ

| Feature | 説明 |
|---------|------|
| `redis` | `k1s0-cache` による `RedisStateStore` を有効化 |
| `mock` | `mockall` によるモック実装を有効化（テスト用） |

**依存追加**: `k1s0-bb-statestore = { path = "../../system/library/rust/bb-statestore" }`

Redis 実装を使用する場合: `k1s0-bb-statestore = { path = "../../system/library/rust/bb-statestore", features = ["redis"] }`

## 関連ドキュメント

- [Building Blocks 抽象化](../_common/building-blocks.md) — BB アーキテクチャ全体設計
- [bb-core 設計](bb-core.md) — BB 基盤クレート
- [StateStore 設計](../../architecture/data/statestore.md) — StateStore Building Block 詳細
