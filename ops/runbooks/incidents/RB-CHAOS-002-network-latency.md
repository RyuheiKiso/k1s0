---
runbook_id: RB-CHAOS-002
title: Chaos Drill — Network Latency（tier1 ↔ data layer 500ms 注入）
category: NET
severity: SEV3
owner: 起案者
automation: argo-workflow
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 即時 / 恒久 5 分
last_updated: 2026-05-02
---

# RB-CHAOS-002: Network Latency Chaos Drill

ADR-TEST-004 5 シナリオ中の 2。tier1 ↔ Valkey / Kafka / PostgreSQL の通信に 500ms latency を注入し、timeout / retry 動作を検証する。詳細は採用後の運用拡大時で完成。

## 1. 前提条件

- LitmusChaos NetworkChaos CRD が利用可能
- tier1 SDK の retry / timeout 設定が proto から生成済（共通規約 §「冪等性と再試行」）

## 2. 対象事象

- 週次 CronChaosEngine スケジュール起動

## 3. 初動手順（5 分以内）

```bash
kubectl apply -f infra/chaos/network-latency/
kubectl get chaosresult -n operation -w
```

## 4. 原因特定手順

drill 失敗時:

- timeout 短すぎ（500ms 注入で SDK timeout 1s 未満）
- retry 設定不備

## 5. 復旧手順

```bash
kubectl delete chaosengine network-latency-tier1 -n operation
```

## 6. 検証手順

- 500ms 注入下で API 成功率 > 95%（retry 込み）
- p99 latency が想定範囲内

## 7. 予防策

- proto の retry policy を見直し、 500ms 環境で動く下限を確立
- Grafana ダッシュボードで latency 分布を継続監視

## 8. 関連 Runbook

- [RB-CHAOS-003 network-partition](RB-CHAOS-003-network-partition.md)
- ADR-TEST-004
