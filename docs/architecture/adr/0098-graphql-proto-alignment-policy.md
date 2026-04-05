# ADR-0098: GraphQL スキーマと Protocol Buffers の整合方針

## ステータス

承認済み

## コンテキスト

k1s0 プラットフォームは GraphQL API（graphql-gateway）と gRPC バックエンドサービスを組み合わせたアーキテクチャを採用している。外部技術監査（2026-04-04）で以下の整合性問題が発見された。

**問題点（CRIT-007）:**

1. `FeatureFlag` GraphQL 型が proto `FeatureFlag` と不整合
   - GraphQL: `key`, `name`, `enabled`, `rolloutPercentage`, `targetEnvironments`
   - Proto: `id`, `flag_key`, `description`, `enabled`, `variants`, `rules`, `created_at`, `updated_at`
   - `description` と `id` フィールドが GraphQL に欠落していた

2. `UpdateTenantInput` GraphQL 型が proto `UpdateTenantRequest` と不整合
   - GraphQL: `name: String`, `status: TenantStatus`
   - Proto: `display_name: String`, `plan: String`
   - `status` 変更は proto では専用 RPC（SuspendTenant/ActivateTenant）に分離されているが GraphQL は混在させていた

## 決定

### FeatureFlag 型の整合

1. GraphQL `FeatureFlag` に `id: ID!` と `description: String` を追加する
2. `name` フィールドは後方互換のために保持し、`flag_key` と同値とする
3. `rolloutPercentage`/`targetEnvironments` は `variants`/`rules` から逆算して維持する（GraphQL クライアントへの後方互換）

### UpdateTenantInput の整合

1. `name: String` を `displayName: String` にリネームして proto `display_name` に整合させる
2. `status: TenantStatus` を削除する（status 変更は `suspendTenant`/`activateTenant` ミューテーションを使用する）
3. `plan: String` を追加して proto `plan` フィールドと整合させる

### 全般方針

- GraphQL スキーマのフィールド名は camelCase（GraphQL 慣例）、proto は snake_case（proto 慣例）
- GraphQL では意味論的に明確な名前を使用し、必要に応じてリゾルバー層で変換する
- proto に存在するフィールドは原則として GraphQL にも公開する（セキュリティ上の理由がある場合を除く）
- proto の enum フィールドは GraphQL でも enum 型として定義する

## 理由

- proto はサービス間通信のソース・オブ・トゥルースであり、GraphQL はそれを外部公開する際のビュー層
- GraphQL スキーマが proto と乖離すると、フロントエンド開発者が正確なデータを取得できない
- `status` を `UpdateTenantInput` から削除することで、Keycloak realm プロビジョニングが絡む複雑な状態遷移を専用ミューテーションに集約できる

## 影響

**ポジティブな影響:**

- フロントエンドが `description` を表示できるようになる
- `UpdateTenantInput` が proto と一致し、実装の意図が明確になる
- GraphQL スキーマが単一の真実の源（proto）に整合する

**ネガティブな影響・トレードオフ:**

- `UpdateTenantInput.name` を使用していた既存クライアントは `displayName` への移行が必要
- `UpdateTenantInput.status` を使用していた既存クライアントは専用ミューテーションへの移行が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 現状維持 | GraphQL と proto の乖離を放置 | 監査指摘を解消できない |
| proto を変更 | proto に `name`/`status` を追加 | proto は gRPC の契約であり、クライアント影響が広い |
| BFF 中間変換 | BFF で変換しつつ両立 | 変換ロジックが複雑化する |

## 参考

- [ADR-0029: GraphQL ゲートウェイアーキテクチャ](0029-graphql-gateway-architecture.md)
- [ADR-0084: GraphQL ページネーション設計](0084-graphql-pagination-design.md)
- `api/proto/k1s0/system/featureflag/v1/featureflag.proto`
- `api/proto/k1s0/system/tenant/v1/tenant.proto`

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（CRIT-007 外部技術監査対応） | @k1s0-team |
