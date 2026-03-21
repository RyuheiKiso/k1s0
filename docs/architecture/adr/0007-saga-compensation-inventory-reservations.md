# ADR-0007: Saga 補償トランザクションのための inventory_reservations テーブル導入

## ステータス

承認済み

## コンテキスト

注文キャンセル時（`order.cancelled` イベント）に在庫を解放する補償トランザクションが未実装だった。
既存の `release_stock` メソッドは `product_id` と `warehouse_id` を引数として受け取るが、
`order.cancelled` イベントには在庫明細情報（`product_id` / `warehouse_id` / `quantity`）が含まれていない。
そのため、`order_id` だけを手がかりに解放対象の在庫予約を特定する仕組みが必要だった。

また、補償トランザクションの冪等性を保証するために、既に解放済みの予約を再処理しない仕組みも必要だった。

## 決定

`inventory_reservations` テーブルを新設し、`reserve_stock` 呼び出し時に同一トランザクション内で予約レコードを挿入する。
`compensate_order_reservations` メソッドをリポジトリトレイトに追加し、`order_id` から予約を逆引きして単一トランザクションで解放する。

## 理由

- **order_id による逆引き**: `order.cancelled` イベントには在庫明細がないため、`order_id` → `inventory_item_id` のマッピングをDBに保持する必要がある
- **単一トランザクション保証**: inventory_items 更新・outbox INSERT・reservations ステータス更新を同一トランザクションで実行し、Outbox パターンの整合性を維持する
- **冪等性**: `status='reserved'` の部分インデックスと `uq_reservation_order_item` UNIQUE 制約により、二重処理と二重予約を防止する
- **非正規化の許容**: `product_id` / `warehouse_id` を reservations テーブルに複製することで、補償時の JOIN を不要にする（パフォーマンスと実装シンプル化のトレードオフ）

## 影響

**ポジティブな影響**:

- `order.cancelled` イベント受信時に自動的に在庫が解放され、手動補償が不要になる
- 補償処理が冪等になるため、Kafka の at-least-once 配信に対して安全
- 単一トランザクションで全操作を完結させるため、部分失敗によるデータ不整合が発生しない

**ネガティブな影響・トレードオフ**:

- `reserve_stock` のトランザクション内で追加 INSERT が発生するため、書き込みレイテンシが微増する
- `product_id` / `warehouse_id` を非正規化して保持するため、在庫アイテムの更新時に reservations テーブルとの乖離が生じうる（補償トランザクションは reservation 作成時点の情報を使用するため実害はない）
- マイグレーション 007 の適用が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A | order.cancelled イベントに在庫明細を含める | イベントスキーマの変更が必要。注文サービスと在庫サービスの結合度が上がる |
| 案 B | order_id → reservation 情報をキャッシュ（Redis 等）で管理 | 外部依存が増える。キャッシュ失効時に補償不可能になるリスク |
| 案 C | 既存の release_stock を order_id 対応に拡張 | order_id から product_id/warehouse_id/quantity を復元する手段がない |

## 参考

- [service-inventory-database.md](../../servers/service/inventory/database.md)
- [service-inventory-implementation.md](../../servers/service/inventory/implementation.md)
- [Saga パターン（マイクロサービスパターン）](https://microservices.io/patterns/data/saga.html)
