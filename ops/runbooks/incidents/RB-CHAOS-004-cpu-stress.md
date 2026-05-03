---
runbook_id: RB-CHAOS-004
title: Chaos Drill — CPU Stress（worker ノード 80% 占有）
category: OPS
severity: SEV3
owner: 起案者
automation: argo-workflow
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 即時 / 恒久 5 分
last_updated: 2026-05-02
---

# RB-CHAOS-004: CPU Stress Chaos Drill

ADR-TEST-004 5 シナリオ中の 4。worker ノードの CPU を 80% 占有し、scheduler の他ノード退避動作を検証する。詳細は採用後の運用拡大時で完成。

## 1. 前提条件

- LitmusChaos StressChaos CRD
- HPA / KEDA 設定済（ADR-SCALE-001）

## 2. 対象事象

- 週次 CronChaosEngine スケジュール起動

## 3. 初動手順（5 分以内）

```bash
kubectl apply -f infra/chaos/cpu-stress/
kubectl top nodes
```

## 4. 原因特定手順

- scheduler 退避が想定どおり動かない場合、PodDisruptionBudget / nodeAffinity 設定確認

## 5. 復旧手順

```bash
kubectl delete chaosengine cpu-stress-worker -n operation
```

## 6. 検証手順

- 80% CPU 占有下で他 Pod の latency が想定範囲内
- HPA / KEDA で scale-out 起動

## 7. 予防策

- nodeAffinity 設定で重要 Pod を特定 zone に配置
- PriorityClass で CPU 優先度を制御

## 8. 関連 Runbook

- [RB-CHAOS-005 disk-io-stress](RB-CHAOS-005-disk-io-stress.md)
- ADR-TEST-004 / ADR-SCALE-001
