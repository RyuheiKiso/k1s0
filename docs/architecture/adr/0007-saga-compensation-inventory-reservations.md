# ADR-0007: Saga 補償トランザクションのための board_columns テーブル導入

## ステータス

承認済み

## コンテキスト

タスクキャンセル時（`task.cancelled` イベント）にボードWIPを解放する補償トランザクションが未実装だった。
既存の `release_wip` メソッドは `board_id` と `column_id` を引数として受け取るが、
`task.cancelled` イベントにはボードWIP明細情報（`board_id` / `column_id` / `quantity`）が含まれていない。
そのため、`task_id` だけを手がかりに解放対象のボードWIPを特定する仕組みが必要だった。

また、補償トランザクションの冪等性を保証するために、既に解放済みのWIPを再処理しない仕組みも必要だった。

## 決定

`board_columns` テーブルを新設し、`reserve_wip` 呼び出し時に同一トランザクション内でWIPレコードを挿入する。
`compensate_task_assignments` メソッドをリポジトリトレイトに追加し、`task_id` からWIPを逆引きして単一トランザクションで解放する。

## 理由

- **task_id による逆引き**: `task.cancelled` イベントにはボードWIP明細がないため、`task_id` → `board_column_id` のマッピングをDBに保持する必要がある
- **単一トランザクション保証**: board_columns 更新・outbox INSERT・WIP ステータス更新を同一トランザクションで実行し、Outbox パターンの整合性を維持する
- **冪等性**: `status='reserved'` の部分インデックスと `uq_wip_task_column` UNIQUE 制約により、二重処理と二重割り当てを防止する
- **非正規化の許容**: `board_id` / `column_id` を board_columns テーブルに複製することで、補償時の JOIN を不要にする（パフォーマンスと実装シンプル化のトレードオフ）

## 影響

**ポジティブな影響**:

- `task.cancelled` イベント受信時に自動的にボードWIPが解放され、手動補償が不要になる
- 補償処理が冪等になるため、Kafka の at-least-once 配信に対して安全
- 単一トランザクションで全操作を完結させるため、部分失敗によるデータ不整合が発生しない

**ネガティブな影響・トレードオフ**:

- `reserve_wip` のトランザクション内で追加 INSERT が発生するため、書き込みレイテンシが微増する
- `board_id` / `column_id` を非正規化して保持するため、ボードカラムの更新時に board_columns テーブルとの乖離が生じうる（補償トランザクションはWIP作成時点の情報を使用するため実害はない）
- マイグレーション 007 の適用が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A | task.cancelled イベントにボードWIP明細を含める | イベントスキーマの変更が必要。タスクサービスとボードサービスの結合度が上がる |
| 案 B | task_id → WIP 情報をキャッシュ（Redis 等）で管理 | 外部依存が増える。キャッシュ失効時に補償不可能になるリスク |
| 案 C | 既存の release_wip を task_id 対応に拡張 | task_id から board_id/column_id/quantity を復元する手段がない |

## 参考

- [service-board-database.md](../../servers/service/board/database.md)
- [service-board-implementation.md](../../servers/service/board/implementation.md)
- [Saga パターン（マイクロサービスパターン）](https://microservices.io/patterns/data/saga.html)
