# DB Corruption Runbook

## Classification
- incident_class: db_corruption
- severity: P1 (page_immediate)
- spec_reference: docs/04_詳細設計/01_適合仕様/18_信頼性運用準備適合仕様.md

## Phase 1: Detection (検知)
1. Prometheus AlertManager で `PostgreSQLDataCorruption` アラートを確認する
2. `pg_catalog.pg_class` 整合性を確認する: `SELECT relname FROM pg_catalog.pg_class WHERE relpages < 0`
3. pgaudit ログで直前の書込操作を特定する
4. 影響テナントを特定する: `SELECT DISTINCT tenant_id FROM domain_event WHERE created_at > NOW() - INTERVAL '10 minutes'`

## Phase 2: Containment (封じ込め)
1. 影響シャードへの書込トラフィックを即座に停止する
   - Istio VirtualService の weight を 0 に変更する: `kubectl patch vs tier2-api --patch '{"spec":{"http":[{"route":[{"destination":{"host":"tier2","weight":0}}]}]}}'`
2. 影響テナントを RLS 追加述語で他テナントから隔離する
3. PagerDuty で on-call DBA に通知する
4. Slack `#k1s0-incidents` チャンネルにインシデント開始を投稿する

## Phase 3: Investigation (調査)
1. PostgreSQL の `pg_catalog.pg_stat_user_tables` で corruption 範囲を確認する
2. Barman で直前の正常バックアップを特定する: `barman list-backups tier2-pg`
3. WAL セグメントを確認して corruption 発生時刻を特定する
4. Jaeger トレースと pgaudit ログを照合して原因操作を特定する

## Phase 4: Resolution (解決)
1. PITR (Point-In-Time Recovery) で corruption 発生直前の時点に復元する
   - `barman-cloud-restore --cloud-provider aws-s3 --bucket-name k1s0-backup tier2-pg <backup_id>`
2. `pg_dump` と `pg_restore` で特定テーブルのみ選択的リストアを試みる
3. staging 環境で復元結果を検証してから本番に適用する
4. audit hash chain の整合性を再検証する

## Phase 5: Recovery Verification (回復確認)
1. SLO ダッシュボードで error rate が閾値以下に回復したことを確認する
2. audit_event テーブルの hash chain を全件検証する
3. 全テナントのデータ整合性を cross-check する
4. 書込トラフィックを段階的に復旧する (10% → 50% → 100%)

## Phase 6: Postmortem (事後分析)
1. 発生から解決までのタイムラインを `docs/postmortem/INC-YYYYMMDD-NNN.md` に記録する
2. 根本原因 (5-Why 分析) を明記する
3. 再発防止アクションを GitHub Issue として登録し担当者を割り当てる
4. pg_partman の retention ポリシーと PITR 保持期間を見直す
5. restore_drill_workflow を再実行して RTO を測定する
