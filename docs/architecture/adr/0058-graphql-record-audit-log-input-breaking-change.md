# ADR-0058: GraphQL RecordAuditLogInput からサーバーサイド取得フィールドを除去

## ステータス

承認済み

## コンテキスト

GraphQL の `RecordAuditLogInput` には `userId`、`ipAddress`、`userAgent` フィールドが含まれていた。
これらはクライアントが任意の値を送信できる状態であり、次のセキュリティリスクが存在していた。

- **userId の偽装**: クライアントが別のユーザー ID を送信することで、他者の行動として監査ログを記録できる。
- **IP アドレスの偽装**: クライアントが任意の IP アドレスを送信することで、実際のアクセス元を隠蔽できる。
- **User-Agent の偽装**: クライアントが任意のブラウザ情報を送信することで、アクセス元の特定が困難になる。

監査ログはセキュリティインシデントの調査や法的証拠として使用されるため、改ざん不能かつ信頼性の高い情報源である必要がある。

## 決定

`RecordAuditLogInput` から `userId`、`ipAddress`、`userAgent` フィールドを削除し、サーバーサイドで取得する。

- **userId**: JWT claims の `sub` フィールドから取得（`GraphqlContext.user_id`）
- **ipAddress**: `X-Forwarded-For` ヘッダーまたはリモートアドレスから取得（`GraphqlContext.ip_address`）
- **userAgent**: `User-Agent` リクエストヘッダーから取得（`GraphqlContext.user_agent`）

`graphql_handler` 関数でリクエストの `HeaderMap` を受け取り、`GraphqlContext` に `ip_address` と `user_agent` を追加する。`record_audit_log` リゾルバでは `input` の代わりに `GraphqlContext` からこれらの値を参照する。

## 理由

- JWT claims はサーバー側で検証済みのため、userId の偽装が不可能になる。
- HTTP ヘッダーはサーバーサイドミドルウェアで取得するため、クライアントが直接制御できない。
- 最小入力の原則（クライアントはサーバーが取得できる情報を送信すべきでない）に従う。

## 影響

**ポジティブな影響**:

- 監査ログの userId/ipAddress/userAgent の信頼性が向上する
- クライアント実装がシンプルになる（送信フィールドが減少）
- セキュリティ監査の要件（H-15）を満たす

**ネガティブな影響・トレードオフ**:

- 破壊的変更（Breaking Change）: 既存の GraphQL クライアントが `userId`/`ipAddress`/`userAgent` を送信している場合は修正が必要
- `X-Forwarded-For` が信頼できるプロキシから送信されていることを前提とする（ロードバランサー設定の確認が必要）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A | フィールドを残しつつサーバー側で無視する | クライアントが誤って値を送信し続けても気づかない。スキーマと実装の不整合が残る |
| 案 B | フィールドを deprecated にして段階的に削除する | 移行期間中もセキュリティリスクが継続する。今回は迅速な対応が求められる |
| 案 C | バッチエンドポイントを新規追加する | 変更範囲が広がり複雑性が増す。既存エンドポイントの修正で十分 |

## 参考

- [GraphQL Security Best Practices - OWASP](https://cheatsheetseries.owasp.org/cheatsheets/GraphQL_Cheat_Sheet.html)
- [ADR-0044: GraphQL Gateway セキュリティ基盤](./0044-graphql-gateway-security.md)
- H-15 外部技術監査指摘事項

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-30 | 初版作成 | @k1s0 |
| 2026-03-30 | 同一監査対応バッチにて GraphQL スキーマの日時フィールド統一を実施（HIGH-A1 対応）。`Job.nextRunAt`, `Job.lastRunAt`, `JobExecution.startedAt`, `JobExecution.finishedAt`, `NotificationLog.sentAt`, `WorkflowTask.dueAt`, `WorkflowTask.decidedAt`, `Session.expiresAt` を `String` から `DateTime` / `DateTime!` に変更。`WorkflowInstance.createdAt` を `String!` から `DateTime!` へ変更。これらも破壊的変更（Breaking Change）であり、既存クライアントの型変換処理の更新が必要。 | @k1s0 |
