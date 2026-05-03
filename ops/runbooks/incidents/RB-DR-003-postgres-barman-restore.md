---
runbook_id: RB-DR-003
title: PostgreSQL barman-cloud restore（経路 C、RTO 15 分）
category: DR
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 15 分 / 恒久 15 分
last_updated: 2026-05-02
---

# RB-DR-003: PostgreSQL barman-cloud restore（DR drill 経路 C）

ADR-TEST-005 経路 C。CloudNativePG barman-cloud で MinIO に保管された base backup + WAL を復元、RPO 数秒 / RTO 15 分。詳細は採用後の運用拡大時で完成。

## 1. 前提条件

- CNPG Operator が cluster で running
- MinIO に base backup（日次）+ WAL（継続）が保管
- ADR-DATA-003 / ADR-DATA-001 と整合

## 2. 対象事象

- PostgreSQL データ破損 / namespace 削除事故
- 検知: tier1-state Get/Set が `5xx`、CNPG `phase: failed`

## 3. 初動手順（5 分以内）

```bash
kubectl get cluster -n cnpg-system -o wide
kubectl logs -n cnpg-system <cnpg-operator-pod> --tail=100
```

Slack `#status` に「経路 C 起動、PostgreSQL restore」を宣言。

## 4. 原因特定手順

- CRD `Cluster.status.conditions` を確認
- `kubectl get backups.postgresql.cnpg.io -n cnpg-system` で base backup 一覧

## 5. 復旧手順

```bash
# 復元先 Cluster manifest を作成（既存と別名で）
kubectl apply -f infra/data/cloudnativepg/restore-cluster.yaml
# CNPG Operator が barman から base + WAL を pull して restore
kubectl wait --for=condition=Ready cluster/<restored-name> -n cnpg-system --timeout=900s
# tier1 の DB connection string を switch
kubectl edit configmap -n tier1-state tier1-config
```

## 6. 検証手順

```bash
kubectl get cluster -n cnpg-system
# tier1-state の Get で復元データ整合確認
make verify-e2e
```

## 7. 予防策

- 日次 base backup の動作確認
- WAL アーカイブ ラグの監視（PromQL: `pg_replication_lag_seconds`）
- 四半期 DR drill 経路 C の継続実施

## 8. 関連 Runbook

- [RB-DR-001 etcd snapshot restore](RB-DR-001-etcd-snapshot-restore.md) — 経路 A
- [RB-DR-002 GitOps full-rebuild](RB-DR-002-gitops-full-rebuild.md) — 経路 B
- ADR-TEST-005 / ADR-DATA-001 / ADR-DATA-003
