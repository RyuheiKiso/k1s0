---
runbook_id: RB-AUD-001
title: 監査証跡ハッシュチェーン整合性 月次検証
category: AUD
severity: SEV1（不整合検出時）
owner: 起案者
automation: argo-workflow
alertmanager_rule: AuditMonthlyVerificationFailure
fmea_id: FMEA-009 系統
estimated_recovery: 暫定 1 時間 / 恒久（調査完了）48 時間
last_updated: 2026-05-02
---

# RB-AUD-001: 監査証跡ハッシュチェーン整合性 月次検証

本 Runbook は監査証跡（`audit_events` テーブル + MinIO Object Lock バケット）に対し、月次バッチでハッシュチェーンを再計算し改ざんを能動検出する手順を定める。
事前検証起点のため [`../incidents/RB-SEC-003-audit-tampering.md`](../incidents/RB-SEC-003-audit-tampering.md)（事後アラート起点）と対をなす。
NFR-H-AUD-001 / NFR-H-COMP-004 / DS-OPS-RB-056 / DS-CF-AUD-* に対応する。

## 1. 前提条件

- 実行者は `compliance-officer` ロール（または `security-sre` で代理）。
- 必要ツール: `kubectl` / `psql` / `mc`（MinIO CLI）/ `sha256sum`。
- kubectl context が `k1s0-prod`。
- 監査 DB: CNPG cluster `k1s0-audit-pg`、namespace `cnpg-system`。
- WORM 保管: MinIO bucket `k1s0-audit`（Object Lock 有効、retention 7 年）。
- 月次バッチジョブ `audit-hash-verifier-monthly` が ArgoWorkflows で configure 済み。

## 2. 対象事象

本 Runbook は以下の起動条件で実行する:

- 月次定期実行（毎月 1 日 03:00 JST、ArgoWorkflows CronWorkflow）。
- 内部監査・外部監査・通報による改ざん疑義の手動起動。

検証範囲は前月分（例: 5 月実行で 4 月 1 日〜30 日のレコード）。

## 3. 初動手順（5 分以内）

月次バッチが ArgoWorkflows で自動起動するため、初動は監視のみ:

```bash
# Workflow 起動状態
kubectl get workflow -n k1s0-audit -l workflow=audit-hash-verifier-monthly

# 検証ジョブの進捗
kubectl logs -n k1s0-audit -l workflow=audit-hash-verifier-monthly -f --tail=100
```

検証完了で不整合が検出された場合、自動的に Slack `#incident-sev1` に通知され、本 Runbook §4 / §5 に進む。

## 4. 原因特定手順（不整合検出時のみ）

```bash
# DB 上のチェーン再計算（先頭 chain_id から）
kubectl exec -n cnpg-system k1s0-audit-pg-1 -- psql -U audit -c "
WITH RECURSIVE audit_chain AS (
  SELECT chain_id, prev_hash, event_hash, payload, ts,
    digest(prev_hash || payload, 'sha256') AS computed_hash
  FROM audit_events
  WHERE ts >= '$(date -d '1 month ago' +%Y-%m-01)' AND ts < '$(date +%Y-%m-01)'
  ORDER BY chain_id ASC LIMIT 1
  UNION ALL
  SELECT a.chain_id, a.prev_hash, a.event_hash, a.payload, a.ts,
    digest(c.computed_hash || a.payload, 'sha256') AS computed_hash
  FROM audit_events a JOIN audit_chain c ON a.chain_id = c.chain_id + 1
)
SELECT chain_id, ts, event_hash, computed_hash
FROM audit_chain
WHERE event_hash != computed_hash
ORDER BY chain_id ASC LIMIT 20;"
```

```bash
# WORM 側との突合（DB と MinIO Object Lock の一致確認）
mc cp --recursive minio/k1s0-audit/$(date -d '1 month ago' +%Y-%m)/ /tmp/worm-audit-$(date +%Y%m)/
diff <(<DB レコード抽出>) <(cat /tmp/worm-audit-*/all-events.jsonl)
```

検出パターン:

| パターン | 観測 | 対処 |
|---|---|---|
| 差し込み（中間に新規レコード） | chain_id ギャップなし、event_hash 不整合が 1 件 | 該当レコードを `audit_quarantine` schema に隔離 |
| 削除（中間レコード喪失） | chain_id ギャップあり | 削除レコードを WORM から復元 |
| 改変（payload 変更） | event_hash と computed_hash が異なる | 該当 chain_id 以降全て連鎖して不整合 |

## 5. 復旧手順（不整合検出時のみ）

### Step 1: SEV1 即時宣言と隔離

```bash
# 該当 chain_id 範囲を audit_quarantine schema に隔離
kubectl exec -n cnpg-system k1s0-audit-pg-1 -- psql -U audit -c "
CREATE SCHEMA IF NOT EXISTS audit_quarantine;
CREATE TABLE audit_quarantine.events_$(date +%Y%m) AS
SELECT * FROM audit_events
WHERE chain_id BETWEEN ${START_CHAIN} AND ${END_CHAIN};"
```

### Step 2: RB-SEC-003 を連鎖発動

不整合確認後は `RB-SEC-003-audit-tampering.md` の §5 復旧手順を実行（フォレンジック保全 + 監督官庁報告判断 + ハッシュチェーン再構築）。

### Step 3: ハッシュチェーン再構築

```bash
# 隔離後、新 chain で audit_events を継続
# 新 chain_id の先頭は前月最後の正常レコードの event_hash を prev_hash として開始
kubectl exec -n cnpg-system k1s0-audit-pg-1 -- psql -U audit -c "
INSERT INTO audit_events (chain_id, prev_hash, event_hash, payload, ts)
VALUES (
  $(<前月最終正常 chain_id> + 1),
  '<前月最終正常 event_hash>',
  digest('<前月最終正常 event_hash>' || '<新 payload>', 'sha256'),
  '<新 payload>',
  NOW()
);"
```

## 6. 検証手順

月次バッチ正常完了の判定基準:

- ArgoWorkflows ジョブ `audit-hash-verifier-monthly` が `Status: Succeeded`。
- 不整合検出件数が 0（`audit_hash_chain_failures_total` が 0）。
- DB 全レコードの再計算ハッシュが記録ハッシュと一致。
- WORM Object Lock 側との突合で全レコード一致。
- 検証結果が `audit_verification_history` テーブルに記録（バッチ実行日時、対象期間、結果）。
- 不整合検出時は `RB-SEC-003` 起動が確認される。

## 7. 予防策

- ポストモーテム起票（不整合検出時のみ、24h 以内、`postmortems/<YYYY-MM-DD>-RB-AUD-001.md`）。
- 検証ジョブの実行頻度を採用後の運用拡大時で月次 → 週次 → 日次に段階的に上げる。
- WORM Object Lock の retention 期間を 7 年で固定（採用検討コミット）。
- audit DB の write 経路を tier1 facade に限定（手動 SQL 実行を Kyverno で禁止）。
- 月次レビューで `audit_verification_history` を確認し、検証ジョブのカバー率を Grafana 可視化。

## 8. 関連 Runbook

- 関連設計書: [`docs/04_概要設計/30_共通機能方式設計/04_監査証跡方式.md`](../../../docs/04_概要設計/30_共通機能方式設計/04_監査証跡方式.md) §DS-CF-AUD-*
- 関連 NFR: [NFR-H-AUD-001 / NFR-H-COMP-004](../../../docs/03_要件定義/30_非機能要件/H_完整性.md)
- 関連 FMEA: [FMEA-009](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)（系統）
- 連鎖 Runbook:
  - [`../incidents/RB-SEC-003-audit-tampering.md`](../incidents/RB-SEC-003-audit-tampering.md) — 事後アラート起点（本 Runbook で不整合検出時に連鎖）
  - [`../incidents/RB-COMP-001-legal-disclosure.md`](../incidents/RB-COMP-001-legal-disclosure.md) — 監督官庁報告
- ArgoWorkflows 定義: `infra/data/audit/audit-hash-verifier-cronworkflow.yaml`
