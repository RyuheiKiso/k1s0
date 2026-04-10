# ADR-0118: event-store publish_failed 再送ジョブの設計

## ステータス

承認済み（Phase A + Phase B 実装完了 2026-04-10）

## コンテキスト

`event-store` は EventSourcing パターンの中核サービスであり、PostgreSQL にイベントを永続化した後、
バックグラウンドタスクで Kafka へイベントを発行する at-least-once 配送を実装している。

Kafka への発行が指数バックオフ 10 回（最大約 5 分）のリトライ後に最終失敗した場合、
これまでのコードはエラーログを出力して `break` するのみで、イベントが完全に消失する問題があった。

LOW-010 監査対応（2026-04-10 一次対応）で `eventstore.events` テーブルに `publish_status` カラム
（`pending` / `published` / `publish_failed`）を追加し、失敗時に `publish_failed` として記録する
仕組みを実装した。

同日の二次監査（2026-04-10）で以下の追加問題が発見された：

1. `mark_events_as_publish_failed` / `mark_events_as_published` 関数が `FORCE ROW LEVEL SECURITY`
   下の UPDATE を行うにも関わらず RLS セッション変数 (`app.current_tenant_id`) を設定していないため、
   UPDATE が常に 0 行となる致命的バグ → **set_config 追加で修正済み**
2. `spawn_publish_failed_monitor` の監視クエリが `FORCE ROW LEVEL SECURITY` 下で
   `app.current_tenant_id` 未設定のまま `SELECT COUNT(*)` を実行するため、RLS により全行が
   フィルタされて常に 0 を返し、監視が機能しない致命的バグ → **SECURITY DEFINER 関数で修正済み**
3. `publish_failed` レコードを回収・再送する再送ジョブが未実装 → **Phase B として実装済み**

### 設計上の課題と解決策

全テナント横断で `publish_failed` レコードを取得・再送するには `FORCE ROW LEVEL SECURITY` を
バイパスする必要がある。PostgreSQL の正規解は **SECURITY DEFINER 関数**パターンである：
- migration 010 でスーパーユーザー権限で SECURITY DEFINER 関数を作成する
- 関数オーナーがスーパーユーザーであれば `FORCE ROW LEVEL SECURITY` の制約を受けない
- アプリロールが当該関数を呼び出すと、スーパーユーザー権限で実行されて全行が参照できる

## 決定

**SECURITY DEFINER 関数 + 統合再送ジョブ**を実装する。

### 実装構成（2026-04-10 実装完了）

#### migration 010: SECURITY DEFINER 関数の追加

`regions/system/database/event-store-db/migrations/010_add_publish_failed_functions.up.sql`

- `eventstore.count_publish_failed_all_tenants()`: 全テナント横断で `publish_failed` 件数を集計
- `eventstore.list_publish_failed_events(p_batch_limit)`: 全テナント横断で `publish_failed` イベントを
  最大 `p_batch_limit` 件取得
- `GRANT EXECUTE ... TO PUBLIC` でアプリロールに実行権限を付与

#### Rust: spawn_publish_failed_retry_job

`regions/system/server/rust/event-store/src/adapter/handler/event_handler.rs`

`spawn_publish_failed_monitor` を廃止し、監視と再送を一体化した `spawn_publish_failed_retry_job` を実装：

1. **Phase A（監視）**: `SELECT eventstore.count_publish_failed_all_tenants()` で件数を取得し、
   0 より大きければ `tracing::warn!` でアラート
2. **Phase B（再送）**: `SELECT * FROM eventstore.list_publish_failed_events(100)` で
   最大 100 件を取得し、stream_id + tenant_id 単位でグループ化して Kafka に再発行
3. **成功時**: `mark_events_as_published` を呼び出し `publish_failed → published` に更新
   （WHERE 句を `IN ('pending', 'publish_failed')` に変更して再送ジョブに対応）
4. **失敗時**: ログを出力して次のインターバル（60 秒後）で再試行

startup.rs から DB が利用可能な場合のみ `spawn_publish_failed_retry_job(pool, publisher)` を起動。

## 理由

SECURITY DEFINER 関数を選択した理由：
- **スキーマ変更最小**: テーブルやロールを変更せず、関数追加のみで実現できる
- **標準的な PostgreSQL パターン**: 特権が必要な操作の委譲に広く使われる確立されたパターン
- **アプリロール分離の維持**: アプリロールに BYPASSRLS を付与する必要がなく、最小権限原則を維持
- **ロールバック可能**: DOWN マイグレーションで関数を削除すれば元の状態に戻る

## 影響

**ポジティブな影響**:

- Kafka 長期障害時（5 分以上）にイベントが消失しなくなる（DB 記録 + 自動再送）
- `tracing::warn!` + Prometheus → Grafana アラートで運用者が失敗を検知できる
- 自動再送により手動介入が原則不要となる
- RLS を維持しながら全テナント横断の操作が可能（SECURITY DEFINER パターン）

**ネガティブな影響・トレードオフ**:

- 60 秒間隔クエリが DB に軽微な負荷を与える（COUNT 集計のみのため実質無視できるレベル）
- SECURITY DEFINER 関数の所有権管理が必要（migration の実行者がスーパーユーザーであること）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: FORCE ROW LEVEL SECURITY を無効化 | `eventstore.events` の RLS を外してクロステナント SELECT を許可 | テナント分離の根幹を破壊するため不可 |
| 案 B: アプリロールに BYPASSRLS を付与 | DB ロールに特権を付与 | アプリロールへの過剰権限。SECURITY DEFINER の方が粒度が細かい |
| 案 C: Kafka DLQ トピックへの転送 | 失敗イベントを Kafka DLQ トピックに送信し dlq-manager が処理 | Kafka 自体がダウンしている状況では DLQ への送信も失敗するため根本解決にならない |
| 案 D: Transactional Outbox パターン | append_events と Kafka publish を同一トランザクションに閉じる | 既存の EventSourcing 設計との大規模な変更が必要。ADR-0119 で長期計画として検討 |
| 案 E: 専用 DB 接続プール（スーパーユーザー） | 再送専用に superuser 接続プールを用意 | アプリ内に superuser 接続を持つことは最小権限に反する |

## 参考

- [LOW-010 監査対応](../../../報告書.md) - 本 ADR の起因となった監査指摘
- [ADR-0116: session-redis JTI fail-open トレードオフ](0116-session-redis-jti-fail-open-tradeoff.md) - セキュリティ vs 可用性のトレードオフ参照
- [lessons.md: PostgreSQL SET LOCAL にパラメータバインドは使えない](../../../tasks/lessons.md)
- migration `009_add_publish_status.up.sql`
- migration `010_add_publish_failed_functions.up.sql`

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-10 | 初版作成（LOW-010 二次対応 + Phase A 実装完了） | kiso ryuhei |
| 2026-04-10 | Phase B 実装完了: SECURITY DEFINER 関数 + 統合再送ジョブ (migration 010, spawn_publish_failed_retry_job) | kiso ryuhei |
