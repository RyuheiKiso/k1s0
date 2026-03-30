# ADR-0054: RLS 段階的実装戦略

## ステータス

承認済み

## コンテキスト

外部監査 H-010、H-012 の指摘により、tenant_id を持つ一部テーブルに Row Level Security（RLS）が未適用であることが判明した。

### 現在の RLS 実装状況

| 状態 | テーブル | 備考 |
|------|---------|------|
| 実装済み | `saga.saga_states`, `saga.saga_step_logs` | migration 008 で追加 |
| 実装済み | `eventstore.*` | migration 006 で追加 |
| tenant_id あり・RLS なし | `auth.api_keys` | migration 012 で tenant_id 追加済み |
| tenant_id あり・RLS なし | `quota.quota_usage` | tenant_id カラムあり |
| tenant_id あり・RLS なし | `session.user_sessions` | migration 003 で tenant_id 追加済み |
| tenant_id なし（対象外） | `notification.*`, `vault.*`, `policy.*`, `config.*` | 別途設計判断が必要 |

### 既存の実装パターン（saga-db より）

`SET app.current_tenant_id` セッション変数を使用した RLS ポリシーで、アプリケーション層からテナント ID を PostgreSQL セッションに設定し、RLS ポリシーが自動的にフィルタリングを行う。

## 決定

以下の 3 テーブルに RLS を追加する。実装パターンは saga-db の既存パターンに準拠する。

### 対象テーブルとマイグレーションファイル

| テーブル | マイグレーションファイル |
|---------|----------------------|
| `auth.api_keys` | `018_add_api_keys_rls.up.sql` |
| `quota.quota_usage` | `005_add_quota_usage_rls.up.sql` |
| `session.user_sessions` | `004_add_user_sessions_rls.up.sql` |

### 実装方針

各マイグレーションで以下を実施する:

1. `ALTER TABLE ... ENABLE ROW LEVEL SECURITY;` でテーブルの RLS を有効化
2. `CREATE POLICY` で `current_setting('app.current_tenant_id')` を使用したテナント分離ポリシーを作成
3. `ALTER TABLE ... FORCE ROW LEVEL SECURITY;` でテーブルオーナーにも RLS を適用

### 対象外

tenant_id を持たないデータベース（`notification.*`, `vault.*`, `policy.*`, `config.*`）については、別 ADR でテナント分離戦略を検討する。

## 理由

- tenant_id カラムが存在するにもかかわらず RLS が未適用のテーブルは、アプリケーション層のバグにより他テナントのデータが漏洩するリスクがある
- DB 層での RLS はアプリケーション層のテナントフィルタリングに対する二重防御として機能する
- saga-db で既に確立されたパターンを再利用することで、実装コストとリスクを最小化できる
- 権限昇格攻撃やアプリケーションバグに対する最終防衛線として DB 層でのテナント分離が必要

## 影響

**ポジティブな影響**:

- マルチテナント環境でのデータ分離が DB 層で保証される
- アプリケーション層のバグによるテナント間データ漏洩リスクが軽減される
- 既存の saga-db パターンとの一貫性が確保される

**ネガティブな影響・トレードオフ**:

- 全クエリの実行前に `SET app.current_tenant_id` が必要（ただし既存パターンで対応済み）
- RLS ポリシーによるクエリプランニングへの軽微な影響
- マイグレーション実行時に既存データとの整合性確認が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| アプリケーション層のみでのテナント分離 | WHERE tenant_id = ? をアプリケーション層で付与 | アプリケーションバグや権限昇格時にデータ漏洩リスクが残る。DB 層での二重防御がない |
| スキーマ分離（テナントごとに別スキーマ） | テナントごとに独立したスキーマを作成 | 既存アーキテクチャとの乖離が大きい。マイグレーション管理が複雑化する |
| 全テーブル一括対応（tenant_id なしテーブル含む） | notification 等を含む全テーブルに RLS を適用 | tenant_id なしテーブルは設計判断が先に必要。段階的対応の方がリスクが低い |

## 参考

- [ADR-0034: マルチテナント設計](0034-multi-tenant-design.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成 | @team |
