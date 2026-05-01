---
runbook_id: RB-SEC-003
title: 監査ログ改ざん検知対応
category: SEC
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: AuditIntegrityFailure
fmea_id: FMEA-009
estimated_recovery: 暫定 1 時間 / 恒久（調査完了）48 時間
last_updated: 2026-05-02
---

# RB-SEC-003: 監査ログ改ざん検知対応

本 Runbook は監査ログのハッシュチェーン整合性検証で改ざんが検知された時の対応を定める。
法令違反リスク（J-SOX、電帳法）が生じる SEV1。
NFR-H-AUD-001 / NFR-H-COMP-004 / FMEA-009 に対応する。
事前検証起点（月次バッチ）は [`../monthly/RB-AUD-001-audit-hash-monthly.md`](../monthly/RB-AUD-001-audit-hash-monthly.md)、本 Runbook は事後アラート起点。

## 1. 前提条件

- 実行者は `security-sre` + Compliance Officer の権限を保持。
- 必要ツール: `kubectl` / `mc`（MinIO CLI）/ `psql` / `sha256sum` / `gpg`（証跡暗号化）。
- kubectl context が `k1s0-prod`。
- 監査証跡の保管先（MinIO Object Lock バケット `k1s0-audit`、CNPG `k1s0-audit-pg`）が稼働中。
- ハッシュチェーン検証ジョブ（`infra/data/audit/hash-chain-verifier-cron.yaml`）が configure 済み。

## 2. 対象事象

- Alertmanager `AuditIntegrityFailure` 発火（ハッシュチェーン検証失敗イベント発生）、または
- 月次バッチ [`RB-AUD-001`](../monthly/RB-AUD-001-audit-hash-monthly.md) の検証で不整合検出、または
- 内部監査・外部監査・通報による改ざん疑義の提起。

検知シグナル:

```promql
# ハッシュチェーン検証失敗カウンタ（1 件で SEV1）
sum(rate(audit_hash_chain_failures_total{namespace="k1s0-tier1"}[5m])) > 0

# 監査ログ書込異常（chain ID 連続性違反）
audit_chain_id_gap_count > 0
```

ダッシュボード: **Grafana → k1s0 Audit Integrity**。
通知経路: PagerDuty `security-sre` → Slack `#incident-sev1` → Compliance Officer 即時。

## 3. 初動手順（5 分以内）

```bash
# 改ざん検知ログの取得
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "hash_chain_mismatch" | json
  | line_format "{{.chain_id}} {{.expected_hash}} {{.actual_hash}}"' \
  --since=1h | head -20
```

```bash
# 監査 DB の状態
kubectl exec -n cnpg-system k1s0-audit-pg-1 -- psql -U audit -c "
  SELECT chain_id, prev_hash, event_hash, ts FROM audit_events
  ORDER BY chain_id DESC LIMIT 10;"
```

```bash
# Object Lock 状態の確認（WORM 保管が機能しているか）
mc legalhold info minio/k1s0-audit/$(date +%Y-%m)/
```

ステークホルダー通知（即時）:

- SEV1 即時宣言、Slack `#incident-sev1` に「監査ログ改ざん検知、Compliance Officer 起動」。
- [`oncall/escalation.md`](../../oncall/escalation.md) を起動、CTO + Compliance Officer + 法務に連絡。
- 調査完了まで該当期間ログの **書込・読出を最小化**（rotation 停止、retention job 停止）。

## 4. 原因特定手順

```bash
# 不整合範囲の特定（先頭から再計算）
kubectl exec -n cnpg-system k1s0-audit-pg-1 -- psql -U audit -c "
WITH RECURSIVE audit_chain AS (
  SELECT chain_id, prev_hash, event_hash, payload,
    digest(prev_hash || payload, 'sha256') AS computed_hash
  FROM audit_events ORDER BY chain_id ASC LIMIT 1
  UNION ALL
  SELECT a.chain_id, a.prev_hash, a.event_hash, a.payload,
    digest(c.computed_hash || a.payload, 'sha256') AS computed_hash
  FROM audit_events a JOIN audit_chain c ON a.chain_id = c.chain_id + 1
)
SELECT chain_id, event_hash, computed_hash
FROM audit_chain
WHERE event_hash != computed_hash
LIMIT 5;"
```

よくある原因:

1. **内部関係者の不正改変**: DB に直接 SQL で `UPDATE` / `DELETE` された。pgaudit ログを確認。
2. **外部攻撃**: SQL Injection / 権限昇格で改ざん。Falco 検知ログを確認。
3. **設定ミスによる権限過剰**: アプリの DB 接続が `audit` ロールで write 可能になっていた。
4. **ハッシュ計算ロジックのバグ**: tier1 audit handler のコード変更で hash アルゴリズム互換性が壊れた（要 staging で再現テスト）。
5. **DB 復元時のオフセット不整合**: 障害復旧時にバックアップから不完全リストア。

エスカレーション: 内部関係者の不正が疑われる場合は CTO + 法務 + 外部法律顧問 + 監督官庁（J-SOX 統制責任者）に連絡。

## 5. 復旧手順

### Step 1: 該当レコードの隔離（〜30 分）

```bash
# 不整合レコードの範囲を特定
START_CHAIN=<最初の不整合 chain_id>
END_CHAIN=<最後の不整合 chain_id>

# 隔離用 schema にコピー
kubectl exec -n cnpg-system k1s0-audit-pg-1 -- psql -U audit -c "
CREATE SCHEMA IF NOT EXISTS audit_quarantine;
CREATE TABLE audit_quarantine.events_$(date +%Y%m%d) AS
SELECT * FROM audit_events
WHERE chain_id BETWEEN ${START_CHAIN} AND ${END_CHAIN};"
```

### Step 2: バックアップとの照合（〜2 時間）

```bash
# MinIO の WORM Object から該当期間の audit log を取得
mc cp --recursive \
  minio/k1s0-audit/$(date +%Y-%m-)<対象月>/ \
  /tmp/audit-restore-$(date +%Y%m%d-%H%M)/

# DB レコードと WORM レコードを diff で照合
diff <(...DB レコード抽出...) <(cat /tmp/audit-restore-*/all-events.jsonl) > /tmp/diff.txt
```

### Step 3: 改ざん範囲のフォレンジック保全

```bash
# Forensics バケットへ保全
mc cp --recursive \
  minio/k1s0-audit/$(date +%Y-%m-)<対象月>/ \
  minio/k1s0-forensics/audit-tamper-$(date +%Y%m%d)/
mc legalhold set minio/k1s0-forensics/audit-tamper-$(date +%Y%m%d)/ ON
```

### Step 4: 監督官庁報告要否判断

- J-SOX 統制違反 → 内部監査責任者経由で監査法人へ報告。
- 個人情報保護法上の漏えい疑い → [`RB-COMP-002`](RB-COMP-002-pii-regulatory-disclosure.md) を並行起動。
- 電帳法違反 → 法務担当が国税庁問合せ要否判断。

### Step 5: ハッシュチェーン再構築（恒久復旧）

```bash
# 隔離 schema を確定後、本番 audit_events に新 chain を追加
# 新 chain の prev_hash = 隔離前最後の正常レコードの event_hash
# 詳細は ../monthly/RB-AUD-001-audit-hash-monthly.md §「ハッシュチェーン再構築」 参照
```

## 6. 検証手順

復旧完了の判定基準:

- ハッシュチェーン検証ジョブが直近実行で `OK`（不整合 0 件）。
- 隔離 schema `audit_quarantine.*` に対象範囲が完全保存されている。
- Forensics バケット `k1s0-forensics/audit-tamper-*` が Legal Hold 設定済み。
- DB 権限（`audit` ロール）が read-only に制限されている（write 経路は tier1 facade のみ）。
- pgaudit ログで該当期間の不正 SQL が特定されている、もしくは「不正アクセス痕跡なし」と結論済み。
- 監督官庁報告（必要な場合）が完了。
- ポストモーテム最終版が起票済み。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`postmortems/<YYYY-MM-DD>-RB-SEC-003.md`）。
- DB アクセス権限の見直し（write 経路を tier1 facade に限定、運用側は read-only）。
- pgaudit の監視ルール強化（INSERT/UPDATE/DELETE on audit_events を即アラート）。
- ハッシュチェーン検証ジョブの実行頻度を上げる（リリース時点 月次 → 採用後の運用拡大時 週次 / 日次）。
- WORM Object Lock の retention 期間延長（`MinIO Lifecycle Policy` で 7 年保管に強化）。
- 月次 Chaos Drill 対象に「audit_events への直接 UPDATE」シナリオを追加（検知ルールの動作確認）。

## 8. 関連 Runbook

- 関連設計書: [`docs/04_概要設計/30_共通機能方式設計/04_監査証跡方式.md`](../../../docs/04_概要設計/30_共通機能方式設計/04_監査証跡方式.md) §DS-CF-AUD-*
- 関連 ADR: [ADR-OBS-002 OTel Pipeline](../../../docs/02_構想設計/adr/ADR-OBS-002-otel.md)
- 関連 NFR: [NFR-H-AUD-001 / NFR-H-COMP-004](../../../docs/03_要件定義/30_非機能要件/H_完整性.md)
- 関連 FMEA: [FMEA-009](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`../monthly/RB-AUD-001-audit-hash-monthly.md`](../monthly/RB-AUD-001-audit-hash-monthly.md) — 事前検証起点、月次バッチ
  - [`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) — PII 含有レコード改ざんの場合
  - [`RB-COMP-001-legal-disclosure.md`](RB-COMP-001-legal-disclosure.md) — 監督官庁報告 / 法的開示
  - [`RB-AUTH-002-auth-abuse-detection.md`](RB-AUTH-002-auth-abuse-detection.md) — 内部 actor 不正の場合
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
