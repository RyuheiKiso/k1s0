# ADR-0062: distributed-lock PostgreSQL 接続クリーンアップ戦略

## ステータス

承認済み

## コンテキスト

外部技術監査により、Go 実装の `PostgresLock` に以下の 2 件の脆弱性が指摘された。

**B-CRIT-02（CRITICAL）: context キャンセル時の接続リーク**

`PostgresLock.Acquire()` は `pg_try_advisory_lock` でロックを取得する際、PostgreSQL の専用コネクション（`*sql.Conn`）を確保し、`Release()` まで保持する。advisory lock はセッションスコープのため、このコネクションをロック保持中は解放できない。

しかし、呼び出し元の context がキャンセルされた場合（タイムアウト、リクエストキャンセル等）、`Release()` が呼ばれないまま処理が中断されると、コネクションが接続プールに返却されない。高負荷環境やタイムアウトが多発するシナリオでは、接続プールが枯渇し全サービスが停止するリスクがあった。

**MED-05（MEDIUM）: nil DB パニック**

`NewPostgresLock(nil)` で構築した `PostgresLock` に対して `Acquire()` を呼ぶと、`l.db.Conn(ctx)` で nil ポインタ参照が発生してパニックとなる。パニックはアプリケーション全体に波及する危険があり、エラーとして返却するべき動作である。

## 決定

以下の 2 つの対応を実装した。

**1. nil DB エラー返却（MED-05 対応）**

`Acquire()` 冒頭に `l.db == nil` チェックを追加し、パニックではなく `errors.New("PostgresLock: db が初期化されていません")` を返す。

**2. context キャンセル時の自動 Release（B-CRIT-02 対応）**

`activeLock` 構造体に `done chan struct{}` を追加した。`Acquire()` 成功後にロック監視 goroutine を起動し、`ctx.Done()` または `done` チャンネルのどちらかを待機する。

- `ctx.Done()` が先に閉じた場合: `context.Background()` を使って `Release()` を自動呼び出しし、コネクションを解放する
- `done` が先に閉じた場合（正常な `Release()` 呼び出し）: goroutine を終了する（cleanup 不要）

`Release()` は `activeLocks` マップからエントリを削除した直後に `close(entry.done)` を呼び出し、監視 goroutine に正常終了を通知する。

## 理由

**goroutine + done チャンネル方式を選択した理由**

advisory lock はコネクションスコープで保持されるため、`Release()` を通じた明示的なアンロックが必要である。単純に context 終了時にコネクションを `Close()` するだけでは、`pg_advisory_unlock` が発行されず lock テーブルにゴーストエントリが残る可能性がある（PostgreSQL はコネクション切断時に advisory lock を自動解放するが、プールのコネクション再利用時に問題が生じる場合がある）。

goroutine による非同期監視は、`Acquire()` の呼び出し元をブロックせずに cleanup を実現できる。done チャンネルにより、正常パスと context キャンセルパスのどちらを経由しても goroutine リークが発生しない。

**`context.Background()` を使って Release する理由**

自動解放時に元の `ctx` を使うと、すでにキャンセルされているため `QueryRowContext` が即座にエラーを返してアンロックに失敗する。`context.Background()` を使うことで、キャンセル後でも確実に `pg_advisory_unlock` を発行できる。

## 影響

**ポジティブな影響**:

- context タイムアウト・キャンセル時の接続プール枯渇リスクを排除する
- nil DB に対して安全なエラー返却を実現し、パニックによるサービス停止を防ぐ
- `Release()` の呼び忘れに対してフォールバックが機能し、運用上の堅牢性が向上する

**ネガティブな影響・トレードオフ**:

- `Acquire()` ごとに 1 つの goroutine が起動するため、高頻度ロック取得時はわずかなメモリオーバーヘッドが生じる（goroutine は `Release()` または context キャンセル時に即座に終了するため長期的な影響はない）
- `activeLock` 構造体のメモリ使用量が `chan struct{}` フィールド分だけ増加する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| コネクション Close のみ | context キャンセル時に `conn.Close()` だけ呼ぶ | `pg_advisory_unlock` が発行されないため、コネクション再利用時にロックが残存する可能性がある |
| finalizer（`runtime.SetFinalizer`） | GC 時にコネクションを解放する | タイミングが不確定で接続プール枯渇の防止として信頼性が低い |
| TTL ベースの定期クリーンアップ | 別 goroutine でタイムアウトしたロックを定期スキャン | advisory lock はキー→コネクション対応が不明確で実装が複雑になる |
| context キャンセルチェックなし（現状維持） | 呼び出し元に Release 責任を委ねる | B-CRIT-02 監査指摘に対応できず、接続プール枯渇リスクが残存する |

## 参考

- [distributed-lock ライブラリ設計](../../../docs/libraries/data/distributed-lock.md)
- [ADR-0050: pg_try_advisory_lock + リトライによる DB マイグレーション排他制御](./0050-advisory-lock-timeout-strategy.md)
- [B-CRIT-02 監査報告書]（外部技術監査 2026-03-31）
- PostgreSQL ドキュメント: [Advisory Locks](https://www.postgresql.org/docs/current/explicit-locking.html#ADVISORY-LOCKS)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-31 | 初版作成（B-CRIT-02・MED-05 監査対応） | @system |
