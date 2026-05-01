---
runbook_id: DAILY-002
title: バックアップ成功確認
category: OPS
severity: 該当なし（定常運用）
owner: 当番 SRE
automation: manual（採用後の運用拡大時で argo-workflow 化検討）
alertmanager_rule: 該当なし（失敗時は BackupJobFailed）
estimated_recovery: 5 分
last_updated: 2026-05-02
---

# DAILY-002: バックアップ成功確認

毎営業日の朝に CNPG / Velero / MinIO Lifecycle のバックアップが過去 24h 以内に成功していることを確認する。

## 1. 前提条件

- 実行者は SRE オンコール当番。
- 必要ツール: `kubectl` / `kubectl cnpg` / `velero` / `mc`。
- kubectl context が `k1s0-prod`。

## 2. 対象事象

毎営業日 09:15 JST 実施（[`morning-health-check.md`](morning-health-check.md) 直後）。

## 3. 初動手順（5 分以内）

```bash
# CNPG backup の最終成功時刻
kubectl cnpg status k1s0-postgres -n cnpg-system | grep "Last successful backup"
kubectl cnpg status k1s0-keycloak-pg -n cnpg-system | grep "Last successful backup"
kubectl cnpg status k1s0-audit-pg -n cnpg-system | grep "Last successful backup"
```

```bash
# Velero backup 一覧
velero backup get | head -10
```

```bash
# MinIO Lifecycle Policy 状態
mc admin info minio
mc ls minio/k1s0-postgres-backup/$(date -d 'yesterday' +%Y-%m-%d)/ | wc -l
```

## 4. 原因特定手順

異常検知時:
1. 24h 超過: [`../incidents/RB-BKP-001-backup-failure.md`](../incidents/RB-BKP-001-backup-failure.md) 起動。
2. 容量不足: MinIO 古い backup 削除。

## 5. 復旧手順

該当 Runbook（`RB-BKP-001`）起動。本 Runbook 自体は確認専用。

## 6. 検証手順

- 全 CNPG cluster の Last successful backup が 24h 以内。
- Velero backup の直近 1 件が `Phase=Completed`。
- MinIO disk 使用率 < 80%。

## 7. 予防策

- 失敗検知時は SEV2 Runbook 即起動。
- 月次容量レビューで容量予測（[`../monthly/dr-drill.md`](../monthly/dr-drill.md) 内）。

## 8. 関連 Runbook

- [`../incidents/RB-BKP-001-backup-failure.md`](../incidents/RB-BKP-001-backup-failure.md)
- [`../../dr/scenarios/RB-DR-001-cluster-rebuild.md`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md)
