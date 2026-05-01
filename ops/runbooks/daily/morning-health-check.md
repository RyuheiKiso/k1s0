---
runbook_id: DAILY-001
title: 朝の死活確認 / 健康診断
category: OPS
severity: 該当なし（定常運用）
owner: 当番 SRE
automation: manual（採用後の運用拡大時で argo-workflow 化検討）
alertmanager_rule: 該当なし
estimated_recovery: 15 分
last_updated: 2026-05-02
---

# DAILY-001: 朝の死活確認 / 健康診断

毎営業日の朝（始業前 15 分）に SRE 当番が実施する全体健康診断手順。
夜間に発生した未通知の問題（アラート閾値以下の劣化、サイレントエラー）を早期発見する。

## 1. 前提条件

- 実行者は SRE オンコール当番。
- 必要ツール: ブラウザ（Grafana）/ `kubectl` / `argocd`。
- kubectl context が `k1s0-prod`。

## 2. 対象事象

毎営業日 09:00 JST に実施。週末・祝日明けは始業時に追加実施。

## 3. 初動手順（5 分以内 = 全体 15 分中の前半）

### Step 1: SLO ダッシュボード確認

Grafana → **k1s0 SLO Overview**:
- API SLO（NFR-A-SLA-001）: 99% 達成
- Audit 完整性 SLO: 100% 達成
- Error budget 消費率: < 5%/h

### Step 2: アラート未対応一覧

```bash
# Alertmanager の active alerts
amtool --alertmanager.url=http://alertmanager.monitoring:9093 alert
```

### Step 3: Pod 異常確認

```bash
kubectl get pods --all-namespaces | grep -vE "Running|Completed"
```

### Step 4: Argo CD 同期状態

```bash
argocd app list | grep -v Synced
```

## 4. 原因特定手順

該当 Runbook を起動:
- Out-of-Sync app → [`../incidents/RB-OPS-002-argocd-out-of-sync.md`](../incidents/RB-OPS-002-argocd-out-of-sync.md)
- Pod 異常 → [`../incidents/RB-API-001-tier1-latency-high.md`](../incidents/RB-API-001-tier1-latency-high.md) など該当 Runbook
- アラート未対応 → 当該アラート ID の Runbook を起動

## 5. 復旧手順

該当 Runbook 起動。本 Runbook 自体は調査専用。

## 6. 検証手順

- 全 Pod が `Running` または `Completed`。
- 全 Argo CD app が `Synced Healthy`。
- Active alert が 0、または対応 Runbook 起動済み。
- SLO error budget 消費率が想定範囲内。

## 7. 予防策

- 異常検出時は該当 Runbook 起動 + Slack `#ops-daily` に共有。
- 週次 SLO レビュー（[`../weekly/slo-burn-rate-review.md`](../weekly/slo-burn-rate-review.md)）に異常パターン蓄積。

## 8. 関連 Runbook

- [`backup-verification.md`](backup-verification.md)
- [`certificate-expiry-check.md`](certificate-expiry-check.md)
- [`error-code-alert-policy.md`](error-code-alert-policy.md)
- [`../weekly/slo-burn-rate-review.md`](../weekly/slo-burn-rate-review.md)
