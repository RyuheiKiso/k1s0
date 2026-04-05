# ADR-0101: GraphQL ページネーション統一方針（Relay Cursor ベース）

## ステータス

承認済み

## コンテキスト

現在の GraphQL スキーマ（`api/graphql/schema.graphql`）では、ページネーション方式が 4 種類混在している。

| エンドポイント | 現在の方式 | 問題 |
|---|---|---|
| `tenants`, `searchAuditLogs` | Relay cursor（`Connection` / `Edge` / `PageInfo`） | 標準準拠 |
| `users`, `jobs`, `workflows` | offset / first | クライアントのキャッシュ整合性が失われる |
| `notifications` | page / pageSize | ページ番号がスケールしない |
| `catalogServices` | 独自 `Connection` 型 | `PageInfo` フィールド構造が標準と異なる |

この不統一は以下の問題を引き起こしている。

1. **クライアント実装の複雑化**: React / Flutter クライアントが各エンドポイントごとに異なるページネーションロジックを実装する必要がある
2. **キャッシュ非効率**: Apollo Client や Relay は Relay cursor 方式に最適化されており、offset ベースでは正規化キャッシュが機能しない
3. **大規模データへの非対応**: offset は全件カウントが必要であり、数千万件規模のデータに対してパフォーマンスが劣化する
4. **拡張性の欠如**: 将来的なリアルタイム更新（Subscription）との統合が困難になる

GraphQL のデファクトスタンダードである [Relay Cursor Connections Specification](https://relay.dev/graphql/connections.htm) を採用し、全エンドポイントを統一することで上記の問題を解消する。

## 決定

新規 GraphQL エンドポイントは Relay Cursor Connections Specification に準拠した方式を採用する。既存エンドポイントは段階的に移行する。

### 共通型定義

すべての `Connection` 型は以下の共通型を使用する。

```graphql
# Relay Cursor 標準 PageInfo 型（既存の PageInfo 型と互換）
type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

# ジェネリック Connection パターン（型ごとに具体化する）
# 例: UserConnection は edges: [UserEdge!]! を持つ
```

各エンティティの Connection 型は `{EntityName}Connection` / `{EntityName}Edge` の命名規則に従う。

```graphql
type UserConnection {
  edges: [UserEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}
type UserEdge {
  cursor: String!
  node: User!
}
```

### クエリシグネチャ

```graphql
# Relay 標準 4 引数を使用する
# first / after: 前進ページネーション
# last / before: 後退ページネーション
users(
  first: Int
  after: String
  last: Int
  before: String
): UserConnection!
```

### 移行計画

- **Phase 1（即時）**: 新規エンドポイントは必ず Relay cursor 方式で実装する
- **Phase 2（2026-Q3）**: `users`, `jobs`, `workflows`, `notifications`, `catalogServices` の既存エンドポイントに cursor 版フィールドを追加し、旧フィールドを `@deprecated` でマークする
- **Phase 3（2026-Q4）**: 旧フィールド（`offset`, `page`, `pageSize`）の利用がゼロになった時点で削除する

## 理由

Relay cursor 方式を選択した根拠は以下のとおりである。

1. **業界標準**: Facebook Relay が提唱し GraphQL Foundation が推奨するデファクトスタンダードである
2. **Apollo Client 最適化**: Apollo Client の `InMemoryCache` は `relayStylePagination` ポリシーにより Relay cursor を自動的に処理でき、手動マージが不要になる
3. **Cursor 安定性**: offset と異なりデータ挿入・削除時にページがずれない。カーソルはエンティティの不変 ID（UUID）をエンコードした値であるため安定している
4. **双方向ページネーション**: `first/after`（前進）と `last/before`（後退）の両方をサポートし、無限スクロールと逆方向ナビゲーションの両方に対応できる
5. **totalCount フィールド**: 必要な場合のみクライアントが要求するフィールドとして設計することで、常に全件カウントクエリが発行されるオーバーヘッドを回避できる

## 影響

**ポジティブな影響**:

- クライアント（React / Flutter）のページネーション実装が統一され、共通コンポーネント化できる
- Apollo Client の自動キャッシュマージにより、クライアント側のボイラープレートが削減される
- 大規模データに対するパフォーマンスが向上する（offset 全件カウント不要）
- 新規エンドポイント追加時の設計ガイドラインが明確になる

**ネガティブな影響・トレードオフ**:

- 既存クライアントコードの移行コストが発生する（Phase 2/3 での破壊的変更）
- サーバーサイドでカーソルのエンコード・デコード実装が必要になる（既存の `tenant.rs` の実装を参考に展開可能）
- `totalCount` の要求がある場合は依然として全件カウントクエリが必要になる
- Phase 2 移行期間中は旧フィールドと新フィールドが並存し、スキーマが冗長になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| offset / limit 統一 | すべての既存実装を offset ベースに統一する | 大規模データでのパフォーマンス問題が解消されない。Apollo Client のキャッシュ最適化が適用されない |
| page / pageSize 統一 | すべてのエンドポイントを page/pageSize 方式に統一する | offset と同じ問題を持つ。GraphQL のエコシステムに合致しない |
| keyset pagination（独自実装） | GraphQL 標準に依存しない独自カーソル実装 | Relay spec と互換性がなくなりクライアントライブラリの恩恵を受けられない |
| 現状維持（混在を許容） | ドキュメント化のみを行い混在を続ける | クライアント実装の複雑化が解消されず、長期的に技術的負債が蓄積する |

## 参考

- [Relay Cursor Connections Specification](https://relay.dev/graphql/connections.htm)
- [Apollo Client relayStylePagination](https://www.apollographql.com/docs/react/pagination/cursor-based/#relay-style-cursor-pagination)
- [GraphQL Cursor Connections Specification](https://graphql.org/learn/pagination/)
- [ADR-0050: Advisory Lock によるフラグ更新排他制御](0050-advisory-lock-flag-update.md)
- [ADR-0083: GraphQL Gateway ドメインモデル設計](0083-graphql-gateway-domain-model.md)
- `regions/system/server/rust/graphql-gateway/src/domain/model/tenant.rs`（既存 Relay cursor 実装の参考実装）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（HIGH-020 監査対応） | @kiso-ryuhei |
