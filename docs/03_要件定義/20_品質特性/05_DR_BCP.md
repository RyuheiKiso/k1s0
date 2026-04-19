# QUA-DRB: 災害復旧 / BCP 要件

RPO、RTO、地理的冗長、BCP 発動条件を定義する。壊滅的障害（Kafka 全損 / etcd 全損 / Istio 全損）の Runbook と連動。

---

## 前提

- [`../../02_構想設計/01_アーキテクチャ/02_可用性と信頼性/`](../../02_構想設計/01_アーキテクチャ/02_可用性と信頼性/)
- [`../30_セキュリティ_データ/07_backup_restore.md`](../30_セキュリティ_データ/07_backup_restore.md)

---

## 要件本体

> 本ファイルは骨格のみ。本文は後続タスクで記述する。
>
> 想定要件 (draft):
>
> - `QUA-DRB-001` RPO（Recovery Point Objective）目標値
> - `QUA-DRB-002` RTO（Recovery Time Objective）目標値
> - `QUA-DRB-003` 地理的冗長の採否判断
> - `QUA-DRB-004` BCP 発動基準
> - `QUA-DRB-005` 復旧訓練の頻度
