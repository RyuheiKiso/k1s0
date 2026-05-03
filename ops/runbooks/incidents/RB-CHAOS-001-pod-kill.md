---
runbook_id: RB-CHAOS-001
title: Chaos Drill — Pod Kill（tier1 service availability）
category: OPS
severity: SEV3
owner: 起案者
automation: argo-workflow
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 即時 / 恒久 5 分
last_updated: 2026-05-02
---

# RB-CHAOS-001: Pod Kill Chaos Drill

ADR-TEST-004（LitmusChaos）の 5 シナリオ最低セット中の 1。tier1 Pod をランダムに削除し Service 継続性 + 自動復旧を検証する。詳細は採用後の運用拡大時で完成。

## 1. 前提条件

- LitmusChaos が `operation` namespace にデプロイ済（採用後の運用拡大時）
- tier1 deployment が `replicas >= 2`（HA 前提）
- Grafana ダッシュボード `chaos-drill.json` 準備済

## 2. 対象事象

- 週次 CronChaosEngine スケジュール起動
- 手動 trigger（kubectl apply -f infra/chaos/pod-kill/）

## 3. 初動手順（5 分以内）

```bash
kubectl get chaosengine -n operation
kubectl get pods -n tier1-state -w
```

Slack `#chaos-drill` に「Pod Kill drill 開始」を投稿。

## 4. 原因特定手順

drill 失敗時（Availability < 99%）:

- Pod 削除タイミングの偏り（ChaosEngine `targetPods.affectedPercentage` 確認）
- HPA / PDB 設定不備（`kubectl get pdb -A`）

## 5. 復旧手順

```bash
# ChaosEngine 停止
kubectl delete chaosengine pod-kill-tier1 -n operation
# 影響 Pod の自動回復を確認
kubectl get pods -n tier1-state
```

## 6. 検証手順

- p99 latency < 200ms 維持（`histogram_quantile(0.99, rate(...))`）
- Error Rate < 1%（5 分平均）
- Pod 削除中の API 連続性（gRPC 失敗率記録）

## 7. 予防策

- HPA 設定で worker 不足を防ぐ
- Pod Disruption Budget 設定で同時削除制限

## 8. 関連 Runbook

- [RB-CHAOS-002 network-latency](RB-CHAOS-002-network-latency.md)
- ADR-TEST-004 / ADR-OPS-001（Chaos Drill 四半期）
