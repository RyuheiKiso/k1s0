---
runbook_id: RB-CHAOS-003
title: Chaos Drill — Network Partition（Valkey / Kafka 切断）
category: NET
severity: SEV3
owner: 起案者
automation: argo-workflow
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 即時 / 恒久 5 分
last_updated: 2026-05-02
---

# RB-CHAOS-003: Network Partition Chaos Drill

ADR-TEST-004 5 シナリオ中の 3。Valkey / Kafka を一時切断し、tier1 のフォールバック動作を検証する。詳細は採用後の運用拡大時で完成。

## 1. 前提条件

- LitmusChaos NetworkChaos CRD（partition mode）
- tier1 のフォールバック設定（Circuit Breaker / Bulkhead）が ADR-TIER1-* で実装済

## 2. 対象事象

- 週次 CronChaosEngine スケジュール起動

## 3. 初動手順（5 分以内）

```bash
kubectl apply -f infra/chaos/network-partition/
```

## 4. 原因特定手順

- Circuit Breaker 不発（fault tolerance 設定不足）
- Cache fallback 経路の bug

## 5. 復旧手順

```bash
kubectl delete chaosengine network-partition-data -n operation
```

## 6. 検証手順

- Partition 中も tier1 公開 API が degraded mode で応答（5xx 連発しない）
- Partition 解除後に自動回復

## 7. 予防策

- 全 service に Circuit Breaker 設定強制（共通 internal/common/circuit_breaker.go）
- Bulkhead 設計を `02_可用性と信頼性/03_グレースフルデグラデーション.md` で継続更新

## 8. 関連 Runbook

- [RB-CHAOS-002 network-latency](RB-CHAOS-002-network-latency.md)
- ADR-TEST-004
