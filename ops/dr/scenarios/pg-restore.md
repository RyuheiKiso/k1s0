# pg-restore — CNPG クラスタ単独リストア（部分復旧シナリオ）

本シナリオは PostgreSQL のデータ喪失（PVC 破損・誤 DELETE・人為ミス）が発生した際に、Barman アーカイブから単一 CNPG cluster をリストアする手順を定める。
RTO 目標: 1 時間。RPO: 6 時間（CNPG backup 周期）。
RB-DR-001（クラスタ全壊）よりも対象範囲が狭く高速。

## 1. 前提条件

- Barman backup が MinIO `k1s0-postgres-backup/` に存在し、6h 以内のバックアップ成功実績がある。
- リストア先 namespace（`cnpg-system`）が利用可能。
- 障害により失われたクラスタ（例: `k1s0-postgres`）は削除済み、または別名で再作成可能。

## 2. シナリオ

例: `k1s0-postgres` が PVC 破損で全データ喪失。Barman 最新バックアップは 2 時間前に成功。
影響: tier1 facade の State / Audit / Secret 一部が利用不可（Audit は別 cluster なので影響なし）。
RPO 侵食: 2 時間分のデータ喪失。RTO 1h で復旧目標。

## 3. 復旧手順

### Step 1: 障害クラスタの削除（〜5 分）

```bash
# 既存 cluster を削除（Barman backup は MinIO 側に残るので問題なし）
kubectl delete cluster k1s0-postgres -n cnpg-system
kubectl delete pvc -n cnpg-system -l cnpg.io/cluster=k1s0-postgres
```

### Step 2: 自動 リストア（CNPG `bootstrap.recovery`）（〜30 分）

```bash
cat <<EOF | kubectl apply -f -
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: k1s0-postgres
  namespace: cnpg-system
spec:
  instances: 3
  bootstrap:
    recovery:
      backup:
        name: k1s0-postgres-backup-latest
  externalClusters:
    - name: k1s0-postgres-backup-latest
      barmanObjectStore:
        destinationPath: s3://k1s0-postgres-backup/
        endpointURL: http://minio.minio.svc:9000
        s3Credentials:
          accessKeyId:
            name: k1s0-minio-credentials
            key: access-key
          secretAccessKey:
            name: k1s0-minio-credentials
            key: secret-key
EOF
```

または事前に `ops/dr/scripts/restore-pg-from-barman.sh --cluster k1s0-postgres` で 1 コマンド化。

### Step 3: リストア進捗の確認

```bash
kubectl get cluster k1s0-postgres -n cnpg-system -w
# Phase=Healthy になれば完了
```

### Step 4: tier1 facade の接続復旧

```bash
kubectl rollout restart deployment/tier1-facade -n k1s0
```

## 4. 検証

- `cnpg_pg_up{cluster="k1s0-postgres", role="primary"} == 1`
- 主要テーブルの行数が想定範囲内（バックアップ時点と一致 or +N行）
- tier1 facade `/healthz` 200
- 直近 10 分のエラーログ 0 件

## 5. RPO 侵食の通知

リストア完了後、影響テナント / 採用組織に対し「2 時間分のデータが喪失」を Slack `#status` で通知。

## 6. 関連

- 関連 Runbook: [`../../runbooks/incidents/RB-DB-002-postgres-primary-failover.md`](../../runbooks/incidents/RB-DB-002-postgres-primary-failover.md), [`../../runbooks/incidents/RB-BKP-001-backup-failure.md`](../../runbooks/incidents/RB-BKP-001-backup-failure.md)
- 関連スクリプト: [`../scripts/restore-pg-from-barman.sh`](../scripts/restore-pg-from-barman.sh)
- 関連設計: `infra/data/cloudnativepg/cluster.yaml`
