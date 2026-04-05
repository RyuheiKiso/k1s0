# ADR-0084: GraphQL カーソルページネーションの安定性

## ステータス

承認済み

## コンテキスト

M-029 外部技術監査にて、GraphQL の一覧クエリで使用している offset-based ページネーションの
安定性に問題があることが指摘された。

現在の実装では、`after: Int` 引数に offset 値を渡す形式でページネーションを実現している。
この方式には以下の問題がある:

- **並行挿入時のズレ**: ページ取得中に新しいレコードが挿入されると、次ページの先頭に
  前ページの末尾レコードが再度出現する（重複）、または一部レコードがスキップされる（欠落）。
- **大量データ時のパフォーマンス**: OFFSET が大きくなるにつれて DB がスキャンする行数が増加し、
  深いページほどクエリが遅くなる。
- **カーソル不透明性**: クライアントが offset 数値を直接扱うため、ページネーション実装が
  クライアントコードに露出する。

影響を受けるクエリ:

- `workflowInstances(first: Int, after: Int)`
- `workflowTasks(..., first: Int, after: Int)`
- `workflows(first: Int, after: Int)`
- その他 `first`/`after` 引数を持つ一覧クエリ全般

## 決定

現時点では offset-based ページネーションの限界をドキュメントに明記し、
中期的に keyset（カーソル）ベースのページネーションへ移行する計画を立てる。

具体的な対応方針:

1. **短期（現状維持）**: offset-based ページネーションを維持しつつ、
   GraphQL スキーマコメントに安定性の制約を明記する。
2. **中期（移行計画）**: `after` 引数を opaque カーソル文字列（Base64 エンコードされた
   keyset 値）に変更する。DB クエリは `WHERE id > :cursor_id ORDER BY id` 形式に移行する。
3. **長期（Relay 準拠）**: Relay Cursor Connections Specification に準拠した
   `Connection` / `Edge` / `PageInfo` 型を導入する。

## 理由

Relay Cursor Connections Specification はページネーションの業界標準であり、
並行挿入に対して安定した動作を保証する。ただし、既存の API を破壊的に変更するコストが高いため、
段階的移行を選択した。

keyset ページネーション（ADR-0082）は既に一部クエリで実装済みであり、
同パターンを WorkflowInstance 等に拡張することで一貫性を確保できる。

## 影響

**ポジティブな影響**:

- 中期移行後、並行挿入下でも重複・欠落のないページネーションを実現できる
- 深いページでのクエリパフォーマンスが向上する
- クライアントコードからページネーション実装詳細が隠蔽される

**ネガティブな影響・トレードオフ**:

- 移行期間中はクライアントとの後方互換性を慎重に管理する必要がある
- keyset ページネーションでは任意ページへのジャンプ（ランダムアクセス）ができない
- 短期的には現状の offset-based の不安定性を許容する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 即時 Relay 準拠移行 | スキーマを Breaking Change で Relay 形式に変更 | 全クライアントの即時対応が必要でリスクが高い |
| offset 維持（現状固定） | offset-based のまま変更しない | 監査指摘の根本解決にならない |
| GraphQL Cursor を UUID 直接使用 | `after: ID` で UUID を渡す | ID の漏洩リスクがあり opaque でない |

## 参考

- [ADR-0082: GraphQL ページネーション統一](0082-graphql-pagination-unification.md)
- [ADR-0083: タスクステータス遷移強制](0083-task-status-transition-enforcement.md)
- [Relay Cursor Connections Specification](https://relay.dev/graphql/connections.htm)
- M-029 外部技術監査指摘事項

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（M-029 監査対応） | @audit-remediation |
