# ADR-0072: RLS ポリシーの tenant_id 型キャスト標準化

## ステータス

承認済み

## コンテキスト

システム全体の `tenant_id` カラムは以下の 3 種類の型で定義されており、統一されていない:

- `UUID`
- `VARCHAR(255)`
- `TEXT`

RLS（Row Level Security）ポリシーの USING 句では `current_setting('app.current_tenant_id', true)`
を使用しているが、この関数は常に `TEXT` 型を返す。カラム型が UUID や VARCHAR の場合、
PostgreSQL は暗黙キャストに依存して比較を行う。

暗黙キャストの問題点:

1. **PostgreSQL バージョン間の動作差異**: 暗黙キャストの挙動はバージョンによって異なる場合があり、
   バージョンアップグレード時に RLS が機能しなくなるリスクがある
2. **実際に発生した不具合（quota-db）**: `quota_usage` テーブルの `tenant_id` カラムに
   `DEFAULT ''`（空文字）が設定されており、アプリケーション設定の
   `app.current_tenant_id = 'system'` と不一致が生じることで、
   system テナントからのアクセスが不能になっていた
3. **マルチテナント境界の信頼性低下**: RLS はセキュリティの要となるため、
   暗黙的な型変換に依存した実装は危険である

対象 DB とマイグレーション番号:

| DB | マイグレーション番号 | 対象テーブル |
|----|---------------------|-------------|
| auth-db | 020 | users, sessions 等 |
| notification-db | 013 | notifications, subscriptions 等 |
| quota-db | 009 | quota_usage（DEFAULT '' も修正） |
| session-db | 005 | sessions 等 |

## 決定

全 RLS USING 句で両辺を明示的に TEXT キャストする形式に統一する:

```sql
-- 修正前（暗黙キャストに依存）
USING (tenant_id = current_setting('app.current_tenant_id', true))

-- 修正後（明示的 TEXT キャスト）
USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
```

あわせて quota_usage テーブルの `tenant_id` カラムの DEFAULT 値を修正する:

```sql
-- 修正前（空文字 DEFAULT はアクセス不能の原因）
ALTER TABLE quota_usage ALTER COLUMN tenant_id SET DEFAULT '';

-- 修正後（system テナントを明示）
ALTER TABLE quota_usage ALTER COLUMN tenant_id SET DEFAULT 'system';
```

各対象 DB に追加マイグレーションファイルを作成し、本変更を適用する。

## 理由

明示的な型キャストを採用した理由:

1. **PostgreSQL バージョン間の移植性**: TEXT キャストを両辺に明示することで、
   バージョン間の暗黙キャスト動作の差異に依存しない安全な比較が実現できる
2. **セキュリティ境界の確実性**: RLS はマルチテナントシステムのセキュリティの
   要となる機能であり、曖昧さのない実装が不可欠
3. **最小変更で最大効果**: 既存の RLS ポリシー構造を維持しつつ、
   キャストを追加するだけで問題が解消できる
4. **将来の型統一を妨げない**: 将来的に tenant_id の型を UUID に統一する場合でも、
   キャストを変更するだけでよい

quota-db の DEFAULT 修正理由:

- 空文字 `''` は有効なテナント ID ではなく、誤って設定されたデフォルト値
- `current_setting('app.current_tenant_id', true)` が `'system'` を返す場合、
  `'' != 'system'` となり system テナントのアクセスがブロックされていた
- `'system'` をデフォルト値にすることで、明示的にテナント ID が設定されない場合でも
  システムテナントとして正しく機能する

## 影響

**ポジティブな影響**:

- マルチテナント境界の信頼性向上（RLS がより堅牢に機能する）
- PostgreSQL バージョンアップグレード時の安全性が高まる
- quota-db の system テナントアクセス不能問題が解消される
- RLS ポリシーの意図が明確になり可読性が向上する

**ネガティブな影響・トレードオフ**:

- 対象 4 DB それぞれに追加マイグレーションを実施する必要がある
- マイグレーション実行中は短時間のロックが発生する可能性がある
  （RLS ポリシーの ALTER は通常 ACCESS EXCLUSIVE ロック不要だが確認が必要）
- 将来的に tenant_id 型を UUID に完全統一した場合は、キャスト記述を見直す必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| tenant_id 型を全 DB で TEXT に統一 | 全テーブルの tenant_id を TEXT 型に変更し、暗黙キャスト不要にする | スキーマ変更の影響範囲が大きい。UUID 型の整合性チェックが失われる。既存インデックスの再作成が必要 |
| current_setting のラッパー関数作成 | `get_tenant_id()` 関数を作成して型変換をカプセル化する | 全 DB に関数を追加する必要がある。RLS ポリシーが直感的でなくなる |
| application_name による tenant_id 受け渡し | SET application_name でテナント ID を渡す | セキュリティ上の懸念（application_name は任意に設定可能）。既存の current_setting 方式からの移行コストが大きい |

## 参考

- [ADR-0012: システム層 RLS スコープ](./0012-system-tier-rls-scope.md)
- [ADR-0054: RLS 段階的実装戦略](./0054-rls-remaining-tenant-tables.md)
- [ADR-0028: マルチテナント ID 取得方式](./0028-tenant-id-acquisition.md)
- [PostgreSQL current_setting 公式ドキュメント](https://www.postgresql.org/docs/current/functions-admin.html#FUNCTIONS-ADMIN-SET)
- [PostgreSQL RLS 公式ドキュメント](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-02 | 初版作成 | kiso ryuhei |
