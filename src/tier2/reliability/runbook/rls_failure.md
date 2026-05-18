# RLS Policy Failure Runbook

## Classification
- incident_class: rls_policy_failure
- severity: P1 (page_immediate) — cross-tenant data leak は最高重大度
- spec_reference: docs/04_詳細設計/01_適合仕様/02_テナント分離適合仕様.md

## Phase 1: Detection (検知)
1. Prometheus アラート `RLSPolicyViolation` を確認する
2. pgaudit ログで cross-tenant アクセスを検出する
   - `SELECT * FROM pgaudit_log WHERE query LIKE '%tenant_id%' AND user_name != 'tier2_rls_admin'`
3. `SET row_security = on` が全接続で強制されているか確認する
4. `pg_policies` ビューで RLS ポリシーが有効であることを確認する
   - `SELECT schemaname, tablename, policyname, permissive, roles, cmd FROM pg_policies`

## Phase 2: Containment (封じ込め)
1. cross-tenant leak が確認された場合: 直ちに全テナントの API アクセスを停止する
2. 影響を受けたテナントに対して即座に通知する (セキュリティインシデントとして扱う)
3. 漏洩したデータの範囲を特定する: 影響テナント × 露出レコード数
4. 法的・コンプライアンス要件に従って当局への報告を検討する

## Phase 3: Investigation (調査)
1. RLS ポリシーが FORCE ENABLE ROW LEVEL SECURITY になっているか確認する
   - `SELECT relname, relrowsecurity, relforcerowsecurity FROM pg_class WHERE relname IN ('domain_event', 'audit_event')`
2. Kyverno ポリシーで `SET row_security = off` が禁止されているか確認する
3. migration が RLS ポリシーを DROP/RECREATE した形跡がないか確認する
4. connection pool (PgBouncer) の tenant_id 設定を確認する

## Phase 4: Resolution (解決)
1. RLS ポリシーを正しい状態に復元する
   ```sql
   ALTER TABLE domain_event FORCE ROW LEVEL SECURITY;
   ALTER TABLE audit_event FORCE ROW LEVEL SECURITY;
   ```
2. 全テーブルの RLS ポリシーを再検証する
3. cross-tenant で露出したデータを audit trail に記録する
4. 影響を受けたテナントのデータを隔離・精査する

## Phase 5: Recovery Verification (回復確認)
1. RLS が正常に機能していることをテストする
   - テナント A のセッションでテナント B のレコードが見えないことを確認する
2. `tier2-rls-check` Litmus テストを実行して回帰確認する
3. 全テナントの audit_event に不正アクセスの痕跡がないことを確認する
4. セキュリティスキャンを再実行する

## Phase 6: Postmortem (事後分析)
1. RLS 無効化の root cause を明記する
2. データ漏洩の全容を法的要件に従い文書化する
3. CI で RLS ポリシーの存在を検証するテストを追加する
4. Kyverno ポリシーで migration による RLS DROP を拒否するルールを強化する
