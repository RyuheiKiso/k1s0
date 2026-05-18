# Audit Hash Chain Mismatch Runbook

## Classification
- incident_class: audit_hash_chain_mismatch
- severity: P1 (page_immediate) — 改ざん検出は最高重大度
- spec_reference: docs/04_詳細設計/01_適合仕様/08_業務エラー監査適合仕様.md

## Phase 1: Detection (検知)
1. Prometheus アラート `AuditHashChainMismatch` を確認する
2. audit ingest gap monitor の heartbeat アラートと照合する (heartbeat.yaml 参照)
3. ClickHouse の audit_event テーブルで hash chain を検証する
   ```sql
   SELECT id, chain_sequence, hash_digest, prev_digest
   FROM audit_event
   WHERE tenant_id = '<affected_tenant>'
   ORDER BY chain_sequence
   LIMIT 100
   ```
4. 不一致が発生した chain_sequence を特定する

## Phase 2: Containment (封じ込め)
1. 影響を受けた audit_event の範囲を確定する (どの chain_sequence から不一致か)
2. 影響テナントへの書込を停止して chain の伸長を防ぐ
3. PostgreSQL の audit_local テーブルと ClickHouse を比較する
4. インシデントを法的・コンプライアンスチームにエスカレーションする

## Phase 3: Investigation (調査)
1. hash_chain.rs の `compute_digest` 実装に変更がなかったか git log で確認する
2. relay.rs の at-least-once 送信ログで重複や欠落がないか確認する
3. PostgreSQL の WAL で audit_event テーブルへの直接書込がなかったか確認する
4. RFC 3161 TSA タイムスタンプ (notarize.sh で生成) と照合する
5. Sigstore transparency log で改ざん記録がないか確認する

## Phase 4: Resolution (解決)
1. 改ざんが確認された場合: セキュリティインシデントとして対応する
   - 直ちに調査チームを招集する
   - 証拠保全のために影響システムをスナップショットする
2. relay バグが原因の場合: relay.rs を修正して chain を再構築する
   - PostgreSQL audit_local を正として ClickHouse を再投影する
3. ClickHouse の ReplacingMergeTree で idempotent 再送信を実行する
4. notarize.sh を再実行して新しい hash root を公証する

## Phase 5: Recovery Verification (回復確認)
1. hash chain の全件検証を実行する
   - `tools/lock_yaml_generator/generate_audit_root_hash.py` で root hash を計算する
2. 修復後の hash root を RFC 3161 + Sigstore で再公証する
3. audit ingest gap monitor の heartbeat が正常に戻ったことを確認する
4. ClickHouse の chain_sequence に欠番や重複がないことを確認する

## Phase 6: Postmortem (事後分析)
1. chain 不一致の root cause を詳細に文書化する
2. 改ざんの可能性がある場合は法的・規制要件に従い当局に報告する
3. audit chain の継続的検証 CI を強化する
4. relay.rs の at-least-once 保証の実装を再審査する
5. heartbeat.yaml の expected_max_gap_seconds を見直す
