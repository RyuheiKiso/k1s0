# minio-tenant-restore — MinIO Tenant 消失からの archive 復元

本シナリオは MinIO Tenant が誤って削除された、または bucket データが喪失した場合の archive 復元手順を定める。
RTO: 2 時間。RPO: 24 時間（MinIO Lifecycle archive 周期）。

## 1. 前提条件

- MinIO archive bucket（`k1s0-archive`）が別リージョンまたは別ストレージクラスに存在し、24h 以内の archive 成功実績がある。
- MinIO Operator が起動中。
- Tenant 設定（access key / secret key）が OpenBao または `infra/data/minio/values.yaml` に保管されている。

## 2. シナリオ

例: `k1s0-postgres-backup` bucket が誤って `mc rm --recursive --force` された。
影響: CNPG Barman archive が利用不可、`RB-BKP-001` トリガー。直前 backup 喪失で RPO 侵食 24h。

## 3. 復旧手順

### Step 1: archive bucket の状態確認

```bash
mc ls minio-archive/k1s0-postgres-backup-archive/$(date -d 'yesterday' +%Y-%m-%d)/ | head -10
```

### Step 2: Tenant の再作成（喪失時のみ）

```bash
kubectl apply -f infra/data/minio/tenant-k1s0.yaml
kubectl get tenant -n minio
```

### Step 3: archive から bucket へ並列 mirror

```bash
# 並列 30 で mirror
mc mirror --parallel 30 --overwrite \
  minio-archive/k1s0-postgres-backup-archive/ \
  minio/k1s0-postgres-backup/
```

または `ops/dr/scripts/restore-minio-from-archive.sh --bucket k1s0-postgres-backup` で 1 コマンド化。

### Step 4: CNPG / Velero に新 bucket を再認識させる

```bash
# CNPG cluster の external storage 確認（変更不要なら何もしない）
kubectl get cluster k1s0-postgres -n cnpg-system -o yaml | grep -A5 barmanObjectStore

# Velero backup location 確認
velero backup-location get
```

## 4. 検証

- `mc ls minio/k1s0-postgres-backup/` で復元データの存在確認。
- CNPG が新 backup を即時取得（手動 trigger）:
  ```bash
  kubectl annotate cluster k1s0-postgres -n cnpg-system \
    cnpg.io/backup-name="post-restore-$(date +%Y%m%d)" --overwrite
  ```
- 24h 後に通常スケジュール backup が成功している。

## 5. RPO 侵食の通知

24h 分のデータが喪失している可能性あり。影響範囲を Slack `#status` で通知。

## 6. 関連

- 関連 Runbook: [`../../runbooks/incidents/RB-BKP-001-backup-failure.md`](../../runbooks/incidents/RB-BKP-001-backup-failure.md)
- 関連スクリプト: [`../scripts/restore-minio-from-archive.sh`](../scripts/restore-minio-from-archive.sh)
