---
runbook_id: RB-CHAOS-005
title: Chaos Drill — Disk IO Stress（PV 書き込み遅延注入）
category: DB
severity: SEV3
owner: 起案者
automation: argo-workflow
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 即時 / 恒久 5 分
last_updated: 2026-05-02
---

# RB-CHAOS-005: Disk IO Stress Chaos Drill

ADR-TEST-004 5 シナリオ中の 5。永続ボリュームに書き込み遅延を注入し、Longhorn / CNPG の PV 異常時動作を検証する。詳細は採用後の運用拡大時で完成。

## 1. 前提条件

- LitmusChaos IOChaos CRD
- Longhorn 3 ノード replication 設定済（ADR-STOR-001）

## 2. 対象事象

- 週次 CronChaosEngine スケジュール起動

## 3. 初動手順（5 分以内）

```bash
kubectl apply -f infra/chaos/disk-io-stress/
kubectl get persistentvolumeclaims -A
```

## 4. 原因特定手順

- Longhorn replication 経路の不整合
- CNPG の WAL 書き込み遅延が `archive_timeout` を超過

## 5. 復旧手順

```bash
kubectl delete chaosengine disk-io-stress-pv -n operation
# Longhorn replica の再 sync 確認
kubectl get volumes.longhorn.io -n longhorn-system
```

## 6. 検証手順

- 書き込み遅延注入下で tier1-state Save が timeout せず success
- PostgreSQL barman-cloud の WAL アーカイブが継続

## 7. 予防策

- Longhorn replication count >= 3 維持
- WAL archive_timeout の設定確認

## 8. 関連 Runbook

- [RB-DR-003 postgres-barman-restore](RB-DR-003-postgres-barman-restore.md)
- ADR-TEST-004 / ADR-STOR-001 / ADR-DATA-001
