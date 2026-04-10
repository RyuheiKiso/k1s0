# ADR-0110: outbox_events テーブルの BYPASSRLS ロール移行ロードマップ

## ステータス

承認済み（実装は将来フェーズ）

## コンテキスト

外部技術監査（HIGH-007）により、activity / board / task の 3 サービスの `outbox_events` テーブルを参照する outbox publisher クエリに `tenant_id IS NULL` 条件が含まれていることが指摘された。

```sql
-- 現在の outbox publisher クエリ（例: task サービス）
SELECT * FROM task.outbox_events
WHERE processed_at IS NULL
  AND tenant_id IS NULL  -- ← この条件が監査で指摘された
ORDER BY created_at
LIMIT 100;
```

この `IS NULL` 条件は ADR-0080 で意図的に設計された動作である。outbox publisher は全テナントのイベントを一括処理するシステムコンポーネントであり、tenant_id によるフィルタリングは不要である。しかし、RLS が有効な状態では publisher の実行ユーザーのテナントスコープに制限されてしまう問題がある。

## 決定

現時点では `tenant_id IS NULL` 条件を維持する（ADR-0080 の設計を継続）。

将来フェーズにおいて `k1s0_outbox_publisher` ロール（BYPASSRLS 権限付き）を導入し、`IS NULL` 条件を削除する移行計画を策定する。

### 移行ロードマップ

**Phase 1（現在）**: `tenant_id IS NULL` 条件で全テナントイベントをフィルタリング

```sql
-- 現在の実装（ADR-0080 設計）
SELECT * FROM task.outbox_events
WHERE processed_at IS NULL AND tenant_id IS NULL
ORDER BY created_at LIMIT 100;
```

**Phase 2（将来）**: `k1s0_outbox_publisher` ロールへ移行

```sql
-- Phase 2 で作成する DB ロール
CREATE ROLE k1s0_outbox_publisher BYPASSRLS;
GRANT SELECT, UPDATE ON task.outbox_events TO k1s0_outbox_publisher;
GRANT SELECT, UPDATE ON activity.outbox_events TO k1s0_outbox_publisher;
GRANT SELECT, UPDATE ON board.outbox_events TO k1s0_outbox_publisher;

-- Phase 2 のクエリ（IS NULL 条件を削除）
SELECT * FROM task.outbox_events
WHERE processed_at IS NULL
ORDER BY created_at LIMIT 100;
```

**影響範囲**: activity / board / task の 3 サービスの outbox publisher コンポーネント。

## 理由

### 現状の IS NULL 条件を維持する理由

1. **ADR-0080 の意図的設計**: outbox publisher は全テナントのイベントを横断的に処理するシステムサービスであり、テナントフィルタリングは機能要件外
2. **実装の安定性**: `IS NULL` 条件は現在正常に動作しており、移行にはマイグレーション・テスト・本番デプロイの調整が必要
3. **段階的移行**: BYPASSRLS ロール導入はセキュリティ影響が大きく、組織の変更管理プロセスに従って実施する必要がある

### BYPASSRLS 移行を将来にする理由

1. **最小権限原則への準拠**: BYPASSRLS は強力な権限であり、本番環境への導入は慎重に計画する必要がある
2. **监査対応**: BYPASSRLS ロールの導入は外部セキュリティ監査の対象となる可能性があり、十分な準備期間が必要
3. **現在の IS NULL 回避策**: 現状の `IS NULL` 条件で機能的には同等の結果を得られており、即時対応の優先度は低い

## 影響

**ポジティブな影響（Phase 2 完了時）**:
- `tenant_id IS NULL` に依存する暗黙的な条件が明示的な BYPASSRLS 設計に置き換わる
- RLS のセマンティクスが明確になり、コードの意図が理解しやすくなる
- IS NULL 条件の誤解（「tenant_id が未設定のレコードのみ取得する」という誤認）を排除できる

**ネガティブな影響・トレードオフ**:
- BYPASSRLS 権限はデータベース全体の RLS をバイパスできる強力な権限であり、誤用リスクがある
- Phase 2 移行時には 3 サービス分のマイグレーション・デプロイ・テストが必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 即時 BYPASSRLS 導入 | Phase 2 を今すぐ実施 | 変更管理プロセスの必要性、テスト期間の確保 |
| outbox_events から RLS を除外 | RLS を DROP して全件アクセス可能に | テナントデータ保護の観点から不適切 |
| publisher を superuser で実行 | superuser ロールで RLS バイパス | 最小権限原則に大きく違反する |

## 参考

- [ADR-0080: outbox IS NULL 設計根拠](0080-outbox-is-null-design.md)
- [ADR-0042: RLS 適用方針](0042-rls-per-database.md)
- [multi-tenancy.md](../multi-tenancy.md) — outbox の注意事項
- 外部技術監査報告書 HIGH-007

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-05 | 初版作成（外部監査 HIGH-007 対応・将来移行ロードマップ策定） | @kiso |
