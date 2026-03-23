# ADR-0021: マイクロサービス間のクロスサービス参照整合性の設計方針

## ステータス

承認済み

## コンテキスト

task サービスの `tasks` テーブルには以下のフィールドが存在する。

```sql
reporter_id   TEXT         NOT NULL,  -- タスクを起票したユーザーID
assignee_id   TEXT,                   -- タスクの担当者ユーザーID（nullable）
```

これらは auth サービスが管理するユーザー ID を参照しているが、`users` テーブルは
auth サービスのデータベースにのみ存在する。task サービスのデータベースには
`users` テーブルが存在しないため、リレーショナルな外部キー制約
（`REFERENCES users(id)`）を追加できない。

この設計上の制約により、以下のリスクが生じる。

1. **孤立レコード**: auth サービスでユーザーが削除された際、task サービスの
   `reporter_id` や `assignee_id` がそのまま残り、実体のない ID を参照し続ける
2. **サービス間結合**: もし task サービスの DB が auth サービスの DB を直接参照する
   外部キーを持つと、両サービスのデプロイ順序・スキーマ変更に依存関係が生じる

k1s0 はマイクロサービスアーキテクチャを採用しており、各サービスが独自の
データベース（Database per Service パターン）を持つ設計原則に従っている。

## 決定

クロスサービスにまたがる外部キー制約（DB レベルの参照整合性）は追加しない。
代わりに、以下 3 つの手段を組み合わせてアプリケーションレベルの整合性を維持する。

1. **アプリケーション層バリデーション**
   - タスク作成時（`CreateTask` gRPC）に auth サービスへ gRPC 問い合わせを行い、
     `reporter_id` および `assignee_id` のユーザー存在を確認する
   - ユーザーが存在しない場合は `NOT_FOUND` エラーを返してタスク作成を拒否する

2. **イベント駆動整合性（Eventual Consistency）**
   - auth サービスのユーザー削除イベント（`UserDeleted`）を Kafka 経由で購読する
   - イベント受信時に task サービスの `assignee_id` を `NULL` に更新する
   - `reporter_id` は NOT NULL 制約があるため、関連タスクのステータスを
     `archived` に変更するか、設計上必要なポリシーを別途決定する

3. **定期整合性チェック（Reconciliation）**
   - バッチジョブを定期実行し、auth サービスに存在しない `reporter_id` /
     `assignee_id` を持つタスクを検出してアラートを発報する
   - 孤立レコードの自動修復は行わず、運用チームによる確認・対処を促す

## 理由

1. **サービス独立性の維持**: Database per Service パターンでは各サービスが
   スキーマ変更・デプロイを独立して行える。クロスサービス FK はこの独立性を破壊する
2. **障害耐性**: auth サービスが一時停止中でも task サービスは既存データの
   読み書きを継続できる。DB レベルの FK があると参照先 DB への接続が必要になる
3. **段階的一致（Eventual Consistency）の受容**: マイクロサービス間の整合性は
   BASE（Basically Available, Soft state, Eventually consistent）モデルで
   管理することがアーキテクチャ上の前提である
4. **実用的な代替手段が存在する**: アプリケーション層バリデーション＋
   イベント駆動更新＋定期チェックの組み合わせにより、運用上十分な整合性レベルを
   達成できる

## 影響

**ポジティブな影響**:

- task サービスと auth サービスを完全に独立してデプロイ・スケールできる
- auth サービスの障害が task サービスの既存データ参照に影響しない
- 将来的にユーザー管理をサービス分割・再編しても DB 制約の影響を受けない

**ネガティブな影響・トレードオフ**:

- アプリケーション層バリデーションにより、タスク作成時に auth サービスへの
  gRPC 呼び出しが追加され、レイテンシが増加する
- ユーザー削除イベントの処理漏れ（Kafka メッセージ損失等）が発生すると
  孤立レコードが残る可能性がある
- 定期整合性チェックの実装・運用コストが発生する
- DB 制約による強制とは異なり、バグ等でバリデーションをバイパスした場合に
  不整合データが混入する可能性がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| クロスサービス外部キー制約を追加 | task DB から auth DB の `users` テーブルを外部参照 | サービス間の DB 結合が生じ、独立デプロイ・スキーマ変更が困難になる。マイクロサービス原則に反する |
| ユーザー情報を task DB に複製 | auth DB の users テーブルを task DB に同期レプリケーション | データの二重管理が生じ、同期ずれによる不整合が常態的に発生するリスクがある |
| Saga パターンで整合性を保証 | タスク作成をトランザクション的に auth サービスと協調処理 | ユーザー参照の確認程度の操作に Saga は過剰設計。複雑性コストに見合わない |

## 参考

- [ADR-0007: Saga 補償トランザクション](./0007-saga-compensation-inventory-reservations.md) — クロスサービストランザクションの設計方針
- [ADR-0018: メッセージング抽象層の統一](./0018-messaging-abstraction-unification.md) — Kafka イベント駆動の基盤方針
- task DB スキーマ: `regions/service/task/database/postgres/migrations/001_create_tasks.up.sql`
- Martin Fowler: [Database per Service](https://microservices.io/patterns/data/database-per-service.html)
- Martin Fowler: [Saga](https://microservices.io/patterns/data/saga.html)
