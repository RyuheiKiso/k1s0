# ADR-0050: pg_try_advisory_lock + リトライによる DB マイグレーション排他制御

## ステータス

承認済み

## コンテキスト

task / board / activity の 3 サービスは DB マイグレーションの多重実行を防ぐために PostgreSQL の advisory lock を使用している。
従来の実装では `pg_advisory_lock()` を使用していたが、この関数はロック取得まで無限に待機するためデッドロック時にサービス起動が永久に停止するリスクがあった。

外部技術監査（HIGH-7）にて「advisory lock に明示的なタイムアウトを設定しないとコンテナ再起動後にマイグレーションが永久ブロックされる可能性がある」と指摘された。

## 決定

`pg_advisory_lock()` を `pg_try_advisory_lock()` + リトライループ + タイムアウトに置き換える。

```rust
// advisory lock をタイムアウト付きで取得（最大 30 秒）
let lock_timeout = std::time::Duration::from_secs(30);
let start = std::time::Instant::now();
loop {
    let locked: (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
        .bind(MIGRATION_LOCK_ID)
        .fetch_one(&mut *migration_conn).await?;
    if locked.0 { break; }
    if start.elapsed() > lock_timeout {
        anyhow::bail!("advisory lock timeout after 30s (lock ID: {})", MIGRATION_LOCK_ID);
    }
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}
```

タイムアウトは 30 秒、ポーリング間隔は 500ms とする。

## 理由

- `pg_try_advisory_lock()` は即時返却（ロック取得失敗時は false）なので非同期ランタイムをブロックしない
- 30 秒のタイムアウトは通常のマイグレーション完了時間（数秒〜十数秒）に対して十分な余裕を持つ
- `statement_timeout` による代替案（後述）はセッション単位で副作用があるため不採用
- ロック取得失敗時は意味のあるエラーメッセージを含むサービス起動失敗とすることで、デバッグ容易性を確保する

## 影響

**ポジティブな影響**:

- デッドロック・プロセスクラッシュによる ロック解放漏れ時に自動回復できる
- タイムアウトエラーが明示的にログに記録されるため障害調査が容易になる
- 非同期ランタイムをブロックしないため他のタスクへの影響がない

**ネガティブな影響・トレードオフ**:

- 30 秒タイムアウトまでサービス起動が遅延する可能性がある（正常時は影響なし）
- ポーリングによるDB接続が発生する（500ms × 最大60回 = 最大60クエリ）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| `pg_advisory_lock()` (従来) | 無限待機でロック取得 | デッドロック時にサービス永久停止のリスク |
| `SET statement_timeout` | セッションのタイムアウトを設定 | 他のクエリにも影響する副作用あり。マイグレーション完了後にリセット必要 |
| `pg_advisory_lock_shared()` | 共有ロックでの待機 | マイグレーションの排他制御に共有ロックは不適切 |
| Kubernetes Job の `activeDeadlineSeconds` | k8s レベルでのタイムアウト | アプリレベルでの制御が優先。加えてローカル開発環境では k8s を使用しない |

## 参考

- [PostgreSQL 公式: pg_try_advisory_lock](https://www.postgresql.org/docs/current/functions-admin.html)
- `regions/service/task/server/rust/task/src/infrastructure/startup.rs`
- `regions/service/board/server/rust/board/src/infrastructure/startup.rs`
- `regions/service/activity/server/rust/activity/src/infrastructure/startup.rs`

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成（HIGH-7 外部技術監査対応） | system |
