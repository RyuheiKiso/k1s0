---
runbook_id: RB-BKP-001
title: Backup 失敗対応（PostgreSQL / Valkey / Kafka / MinIO）
category: BKP
severity: SEV2
owner: 協力者
automation: manual
alertmanager_rule: BackupJobFailed
fmea_id: 間接対応
estimated_recovery: 暫定 1 時間 / 恒久 24 時間
last_updated: 2026-05-02
---

# RB-BKP-001: Backup 失敗対応

本 Runbook は PostgreSQL / Valkey / Kafka / MinIO のいずれかのバックアップジョブが失敗した時の対応を定める。
RPO（24h）侵食リスクを伴うため SEV2。NFR-A-REC-001 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + MinIO admin 権限。
- 必要ツール: `kubectl` / `kubectl cnpg` / `mc`（MinIO CLI）。
- kubectl context が `k1s0-prod`。
- Backup スケジュール: CNPG Barman 6h 間隔、Velero 24h、MinIO Lifecycle 24h。

## 2. 対象事象

- Alertmanager `BackupJobFailed` 発火（`backup_status{result="failed"} > 0` を 30 分継続）、または
- CNPG `kubectl get backup -n cnpg-system` で `STATUS=Failed`。

検知シグナル:

```promql
# CNPG backup 失敗
cnpg_backup_status{phase="failed"} > 0

# MinIO Lifecycle Job 失敗
minio_lifecycle_failed_total > 0

# 最終成功時刻からの経過時間（24h で SEV2、48h で SEV1 昇格）
time() - max(cnpg_last_successful_backup_timestamp_seconds) > 86400
```

ダッシュボード: **Grafana → k1s0 Backup Health**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#alert-data`。

## 3. 初動手順（5 分以内）

```bash
# CNPG backup の状態
kubectl get backup -n cnpg-system --sort-by=.metadata.creationTimestamp | tail -10
kubectl describe backup <failed-backup-name> -n cnpg-system

# Velero backup の状態
velero backup get | head -10
velero backup describe <failed-backup-name>

# MinIO のディスク使用率（容量起因の可能性）
mc admin info minio
```

ステークホルダー通知: SEV2 のため Slack `#alert-data` に「<対象> backup 失敗、調査中」を投稿。
24h 連続失敗で SEV1 昇格、`oncall/escalation.md` 起動。

## 4. 原因特定手順

```bash
# CNPG operator のログ
kubectl logs -n cnpg-system deploy/cnpg-controller-manager --tail=100 | grep "backup"

# Velero ログ
velero backup logs <failed-backup-name>

# MinIO サーバログ
kubectl logs -n minio statefulset/minio --tail=100 | grep -iE "ERROR|backup"
```

よくある原因:

1. **MinIO ディスク満杯**: backup 保管先の容量不足。`mc admin info` で確認。
2. **認証失敗**: MinIO credential が rotate されたが backup 設定に反映されていない。OpenBao の secret 確認。
3. **ネットワーク到達性**: NetworkPolicy 変更で backup ジョブが MinIO に届かない。
4. **backup ジョブ自体のエラー**: Operator バグ、Pod OOM、PVC スナップショット失敗。
5. **schedule 重複**: 前回 backup が 6h 内に終わらず次が起動できない。

## 5. 復旧手順

### Step 1: 手動 backup の即時実行

```bash
# CNPG: 手動 backup を即時起動
cat <<EOF | kubectl apply -f -
apiVersion: postgresql.cnpg.io/v1
kind: Backup
metadata:
  name: k1s0-postgres-manual-$(date +%Y%m%d-%H%M)
  namespace: cnpg-system
spec:
  cluster:
    name: k1s0-postgres
EOF

# Velero: 手動 backup
velero backup create k1s0-manual-$(date +%Y%m%d-%H%M) \
  --include-namespaces k1s0,k1s0-tier1,k1s0-data
```

### Step 2: 容量起因なら MinIO クリーンアップ

```bash
# 古い backup を確認
mc ls minio/k1s0-postgres-backup/ | head -20
# 90 日超の backup を削除（Lifecycle Policy 失敗時の手動）
mc rm --recursive --force \
  minio/k1s0-postgres-backup/$(date -d '90 days ago' +%Y-%m)/
```

### Step 3: 認証起因なら secret rotation

```bash
# OpenBao の secret 確認
kubectl get secret -n cnpg-system minio-credentials -o yaml
# 必要なら CNPG cluster の external storage credential を更新
```

### Step 4: 定期スケジュールの再有効化

```bash
# CronJob が suspended 状態なら再開
kubectl patch cronjob cnpg-backup-schedule -n cnpg-system \
  --type=merge -p '{"spec":{"suspend":false}}'
```

### Step 5: RPO 侵食の判定

```bash
# 最終成功 backup の時刻確認
kubectl cnpg status k1s0-postgres -n cnpg-system | grep "Last successful backup"

# 24h を超えていれば SEV1 昇格
```

## 6. 検証手順

復旧完了の判定基準:

- 直近の手動 backup が `STATUS=Completed`。
- `cnpg_last_successful_backup_timestamp_seconds` が 6h 以内。
- 次回スケジュール backup が成功（次の周期で確認）。
- MinIO の disk 使用率が 80% 未満。
- Velero の `backup get` で `Phase=Completed`。
- 直近 24h で `BackupJobFailed` アラートが再発火していない。

## 7. 予防策

- ポストモーテム起票（72 時間以内、`postmortems/<YYYY-MM-DD>-RB-BKP-001.md`）。
- MinIO Lifecycle Policy の見直し（保管期間と容量のバランス）。
- backup 容量予測の月次レビュー（`weekly/backup-verification.md` で実施）。
- 採用後の運用拡大時 で MinIO クロスリージョンレプリケーション導入（RPO 24h → 1h 短縮）。
- NFR-A-REC-001 / NFR-A-REC-002 の MTTR ログを更新。

## 8. 関連 Runbook

- 関連 NFR: [NFR-A-REC-001（RTO 4h）](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 連鎖 Runbook:
  - [`RB-DB-002-postgres-primary-failover.md`](RB-DB-002-postgres-primary-failover.md)
  - [`RB-DR-001-cluster-rebuild.md`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md) — backup が無いと DR 不可
  - [`../daily/backup-verification.md`](../daily/backup-verification.md) — 日次検証
- 関連 daily 運用: `../daily/backup-verification.md`
