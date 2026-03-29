# ADR-0043: Service Tier GraphQL 統合方針

## ステータス

承認済み

## コンテキスト

k1s0 の現行 GraphQL Gateway（`graphql-gateway` サーバー）は System Tier のデータ
（Tenant、FeatureFlag、Config、Navigation、AuditLog、JWKS 等）を一元的に公開する。

Service Tier（task、board、activity 等）は現在 REST/gRPC の個別エンドポイントを持ち、
クライアント（React SPA、Flutter アプリ）は各サービスを直接呼び出している。

外部技術監査（M-13）において、「Service Tier を GraphQL Gateway に統合することで
クライアント側のオーケストレーション負荷を軽減できるのではないか」という観点から
統合方針の明示が求められた。

### 現状の技術スタック

- `api/graphql/schema.graphql`: System Tier 向けスキーマ定義
- `api/proto/k1s0/service/`: task/board/activity 向け proto 定義
- GraphQL Gateway は async-graphql (Rust) のコードファーストアプローチで実装

### 検討された問題点

1. **N+1 問題**: Service Tier の各エンティティは board/task/activity の連携クエリを含み、
   DataLoader 実装なしに統合すると N+1 クエリによるパフォーマンス劣化が想定される
2. **認証スコープの違い**: System Tier は全テナントを横断するシステム操作、
   Service Tier はテナント内ビジネスロジックであり、権限モデルが異なる
3. **スキーマ複雑性**: board/task/activity の型定義を単一スキーマに統合すると
   スキーマサイズが大幅に増加し、型生成・メンテナンスコストが上昇する
4. **開発チーム独立性**: Service Tier のスキーマ変更が Gateway のリリースサイクルに依存するようになる

## 決定

**Service Tier の GraphQL 統合は現時点（2026-Q1）では非対応とする。**
統合の是非については 2026-Q3 以降に改めて検討する。

当面は以下の方針を維持する：

- Service Tier（task/board/activity）は gRPC/REST エンドポイントのみを公開する
- クライアント側のオーケストレーションは BFF-Proxy または React Query/Flutter Riverpod で吸収する
- GraphQL Gateway の責務は System Tier データの集約に限定する

## 理由

### 現時点で統合しない根拠

1. **DataLoader 未実装**: N+1 問題を解決するための DataLoader が未実装であり、
   統合後のパフォーマンスが保証できない
2. **認証分離の維持**: System Tier と Service Tier の権限モデルを単一 Gateway に
   混在させることで、権限チェックのミスによるデータ漏洩リスクが生じる
3. **開発速度**: Service Tier は現在積極的に機能追加中であり、
   Gateway 統合のオーバーヘッドが開発速度を低下させる
4. **スキーマ増大リスク**: `schema.graphql` の肥大化は型生成・クライアントバンドルサイズに影響する

### 2026-Q3 以降の再検討条件

以下の条件が揃った段階で統合を再検討する：

| 条件 | 基準 |
|------|------|
| DataLoader 実装 | 全 Service Tier クエリで N+1 が解消されること |
| Federation 対応 | Apollo Federation または Schema Stitching の導入評価完了 |
| 権限モデル整理 | System/Service Tier の権限境界の仕様確定 |
| クライアント需要 | React/Flutter クライアントからの統合要望が明確化 |

## 影響

**ポジティブな影響**:

- 現行のシンプルな Gateway 実装を維持できる
- Service Tier の独立したリリースサイクルが保たれる
- N+1 問題のリスクなしに開発を継続できる

**ネガティブな影響・トレードオフ**:

- クライアントが複数のエンドポイント（GraphQL + REST）を使い分ける必要がある
- BFF-Proxy でのオーケストレーションが増加する可能性がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: 即時統合 | Service Tier を現行 Gateway に追加 | DataLoader 未実装のため N+1 問題が未解決 |
| 案 B: Apollo Federation | Subgraph として分散 Schema を構成 | 導入・運用コストが高い。現フェーズには過剰 |
| 案 C: GraphQL Mesh | 既存 REST/gRPC を自動 GraphQL 変換 | 型安全性が低下。パフォーマンスが不透明 |

## 参考

- [api/graphql/schema.graphql](../../../api/graphql/schema.graphql)
- [api/proto/k1s0/service/](../../../api/proto/k1s0/service/)
- [docs/servers/system/graphql-gateway/server.md](../../servers/system/graphql-gateway/server.md)
- 外部技術監査報告書 M-13: "Service Tier GraphQL 統合方針の明示を求める"

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-28 | 初版作成（M-13 監査対応） | 監査対応チーム |
