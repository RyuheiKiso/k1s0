# ADR-0093: tenant_id 型統一 — featureflag-db・config-db の UUID→TEXT マイグレーション

## ステータス

承認済み

## コンテキスト

k1s0 プロジェクトでは全データベースの `tenant_id` カラムに `TEXT` 型を採用することを標準としている（例: workflow-db, session-db, notification-db 等）。しかし外部技術監査（C-004）により、以下の 2 データベースが `UUID` 型のままであることが判明した。

- `regions/system/database/featureflag-db/migrations/004_add_tenant_id.up.sql`
  - `feature_flags.tenant_id UUID NOT NULL`
  - `flag_audit_logs.tenant_id UUID NOT NULL`
- `regions/system/database/config-db/migrations/010_add_tenant_id.up.sql`
  - `config_entries.tenant_id UUID NOT NULL`
  - `config_change_logs.tenant_id UUID NOT NULL`

PostgreSQL の `current_setting('app.current_tenant_id', true)` は常に `TEXT` を返す。UUID 型のカラムと比較する際には暗黙キャスト、または `::TEXT` による明示的キャストが必要になり、RLS ポリシーにも `::TEXT` キャストが記述されている（`featureflag/005_add_rls.up.sql`）。これは型不一致による暗黙のキャストコストや、将来の RLS ポリシー記述での誤りを招くリスクがある。

また、テナント ID として UUID 形式に限らず任意の文字列識別子（例: `'system'`, `'default'`, サービス名等）を使用するユースケースが存在する。UUID 型ではこれらの非 UUID テナント ID を格納できない。

## 決定

featureflag-db および config-db の `tenant_id` カラムを `UUID` 型から `TEXT` 型に変更する新規マイグレーションを追加する。

- featureflag-db: `006_alter_tenant_id_to_text.up.sql` / `006_alter_tenant_id_to_text.down.sql`
- config-db: `012_alter_tenant_id_to_text.up.sql` / `012_alter_tenant_id_to_text.down.sql`

既存の UUID 値は `USING tenant_id::TEXT` により透過的に文字列表現（`'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'` 形式）へ変換される。カラムのデフォルト値は他サービスと合わせて `'system'` に統一する。

**既存マイグレーションファイルは変更しない**（不変性原則）。

## 理由

1. **型統一による暗黙キャスト排除**: 全サービスが `TEXT` 型を使用することで、RLS ポリシーの `::TEXT` キャストが不要になり、クエリの意図が明確になる
2. **将来の柔軟性**: UUID 形式以外のテナント ID（`'system'`, `'default'` 等）への対応が可能になる
3. **後方互換性**: 既存の UUID 値は文字列として透過的に保持されるため、アプリケーション側の変更は不要
4. **標準への準拠**: workflow-db・session-db 等の先行実装と型を統一し、プロジェクト全体のデータモデルの一貫性を高める

## 影響

**ポジティブな影響**:

- RLS ポリシーの記述が簡潔になる（`tenant_id = current_setting(...)::TEXT` で十分）
- `current_setting()` の戻り値との型不一致が解消され、インデックス効率が向上する
- テナント ID の柔軟性が増し、非 UUID 形式の識別子にも対応可能になる
- featureflag-db と config-db が全サービス標準の `TEXT` 型に準拠する

**ネガティブな影響・トレードオフ**:

- 既存データの型変換（`::TEXT` キャスト）が必要なため、マイグレーション実行時に短時間のテーブルロックが発生する
- ロールバック時（down.sql）は、TEXT 値が有効な UUID 形式でない場合は失敗する（`'system'` 等の非 UUID 値が存在する場合）
- featureflag-db の既存 RLS ポリシー（`005_add_rls.up.sql`）の `::TEXT` キャストは後方互換性のため残存する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 現状維持（UUID のまま） | `::TEXT` キャストで対応し続ける | 型不一致が残り、将来の誤りリスクや非 UUID テナント ID への非対応が継続する |
| featureflag-db のみ変更 | featureflag-db だけ TEXT に変更する | config-db も同じ問題を抱えており、部分的な対応では一貫性が得られない |
| 既存マイグレーション修正 | `004_add_tenant_id.up.sql` を直接修正する | マイグレーションファイルの不変性原則に反し、適用済み環境との乖離を生む |

## 参考

- [ADR-0050: sqlx::QueryBuilder への移行](0050-sqlx-query-builder-migration.md)
- `regions/system/database/workflow-db/migrations/008_add_tenant_id_and_rls.up.sql` — TEXT 型採用の先行実装
- `regions/system/database/featureflag-db/migrations/005_add_rls.up.sql` — `::TEXT` キャストが必要だった経緯
- 外部技術監査レポート C-004（2026-04-04）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（C-004 監査対応） | @kiso-ryuhei |
