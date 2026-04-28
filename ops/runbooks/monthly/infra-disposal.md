# インフラ廃棄・暗号消去手順 Runbook

> **alert_id**: ops.infra.disposal.monthly-review
> **severity**: SEV3
> **owner**: sre-ops
> **estimated_mttr**: 120m（計画作業、緊急度なし）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

本 Runbook は定期実行（月次）および以下のトリガーで実行する。

- 月次インフラレビューの結果、廃棄対象 VM / ストレージが特定された場合
- Node の retire 予定がクラウドプロバイダから通知された場合
- セキュリティ監査で不要リソースの存在が指摘された場合
- PVC の reclaimPolicy が Delete に設定されていたことが判明し、手動消去が必要な場合

確認ダッシュボード:

```promql
# 未使用 PVC の特定（Bound でない PVC）
kube_persistentvolumeclaim_status_phase{phase!="Bound", namespace=~"k1s0.*"} == 1

# 長期間 Pod にマウントされていない PV（7 日以上 Released）
kube_persistentvolume_status_phase{phase="Released"} == 1
```

## 2. 初動 (Immediate Action, 〜15 分)

廃棄対象リソースの棚卸しを行う（月次定期作業）。

- [ ] 廃棄対象の VM / ストレージ一覧を作成する

  ```bash
  # Kubernetes Node の一覧（廃棄候補に cordon 済みのものを確認）
  kubectl get nodes -o custom-columns="NAME:.metadata.name,STATUS:.status.conditions[-1].type,TAINTS:.spec.taints"

  # 解放済み PV の一覧
  kubectl get pv -o custom-columns="NAME:.metadata.name,STATUS:.status.phase,CLAIM:.spec.claimRef.name,CLASS:.spec.storageClassName" | grep -v Bound
  ```

- [ ] 廃棄前に 4-eyes 承認を取得する

  Slack `#sre-ops` で廃棄対象リストと承認者（SRE 2 名）を明記したスレッドを作成する。

- [ ] データバックアップの最終確認を行う（廃棄対象がデータを持つ場合）

  ```bash
  # CNPG バックアップ最終実行時刻の確認
  kubectl cnpg status k1s0-postgres -n cnpg-system | grep -i "last.*backup"

  # MinIO バケットの内容確認
  kubectl exec -n minio deploy/minio -- \
    mc ls local/k1s0-postgres-backup/ | tail -5
  ```

## 3. 復旧 (Recovery, 〜60 分)

「復旧」ではなく **廃棄実施**手順として記載する。

### VM（Kubernetes Node）の廃棄

```bash
# 1. Node を cordon（新規 Pod のスケジュールを停止）
kubectl cordon <node-name>

# 2. Node を drain（既存 Pod を退避）
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data --timeout=300s

# 3. Node を削除（Kubernetes から登録解除）
kubectl delete node <node-name>

# 4. Cloud Provider で VM を停止・削除
# GCP の場合:
gcloud compute instances delete <instance-name> --zone <zone> --quiet
# AWS の場合:
aws ec2 terminate-instances --instance-ids <instance-id>
```

### PVC / PV の廃棄と暗号消去

```bash
# 1. PVC を削除（reclaimPolicy=Retain の場合 PV は Released になる）
kubectl delete pvc <pvc-name> -n <namespace>

# 2. Released PV の reclaim policy を Delete に変更して消去
kubectl patch pv <pv-name> -p '{"spec":{"persistentVolumeReclaimPolicy":"Delete"}}'
kubectl delete pv <pv-name>
```

**暗号消去（Crypto Erase）の実施**:

k1s0 はすべての PV に対して StorageClass レベルの暗号化（クラウドプロバイダ KMS または LUKS）を有効にしている。暗号化済みボリュームの廃棄手順:

```bash
# GCP Persistent Disk（CMEK 使用の場合）
# KMS 鍵を無効化することで暗号消去と同等の効果を得る
gcloud kms keys versions disable <key-version> \
  --key <key-name> --keyring <keyring-name> --location <region>

# AWS EBS（CMK 使用の場合）
# 鍵を削除スケジュールに設定
aws kms schedule-key-deletion --key-id <key-id> --pending-window-in-days 7
```

**物理メディア廃棄（オンプレ / ベアメタルの場合）**:

```bash
# LUKS 暗号化ボリュームの鍵消去（cryptsetup erase）
cryptsetup erase /dev/<device>
# または NIST SP 800-88 準拠の上書き
shred -vfz -n 3 /dev/<device>
```

### OpenBao の不要 Secret / Policy の削除

```bash
# 廃棄対象サービスの Secret を削除
bao kv delete secret/k1s0/<service-name>

# 対応する Policy を削除
bao policy delete k1s0-<service-name>-policy

# AppRole の削除
bao auth disable approle/<service-name>
```

## 4. 原因調査 (Root Cause Analysis)

本 Runbook は計画作業であり「原因調査」は不要な場合が多いが、以下のケースで調査を行う。

**廃棄作業中に予期しないエラーが発生した場合のチェック項目**:

1. **PVC が削除できない（Finalizer が残留）**: `kubectl get pvc -o yaml | grep finalizer` で `kubernetes.io/pvc-protection` が残っていないか確認。`kubectl patch pvc <name> -p '{"metadata":{"finalizers":null}}'` で強制削除。
2. **Node が Drain できない（PDB がブロック）**: `kubectl get pdb -A` で PodDisruptionBudget を確認。対象 Pod の PDB を一時的に削除してから drain する。
3. **KMS 鍵の削除が拒否される**: 鍵が他のリソースで使用中。Cloud Provider のコンソールで依存リソースを確認する。
4. **bao コマンドでの Permission Denied**: SRE の AppRole Token の権限が不足。`bao token lookup` で権限を確認し、必要であれば admin token で実行する。

## 5. 事後処理 (Post-incident)

- [ ] 廃棄完了証跡を `ops/runbooks/postmortems/disposal-log-<YYYY-MM>.md` に記録（audit 要件）
- [ ] Cloud Provider のコスト管理ダッシュボードで廃棄後のコスト削減を確認
- [ ] 不要リソースが残留していないか確認（翌日に再チェック）
- [ ] 廃棄対象が CMDB や資産管理台帳に登録されていた場合は削除申請を行う
- [ ] セキュリティ監査へ暗号消去証明書（vendor 発行）を提出する（該当する場合）

## 関連

- 関連設計書: `infra/k8s/storage/storage-classes.yaml`、`infra/security/openbao/values.yaml`
- 関連 ADR: `docs/02_構想設計/adr/ADR-SEC-002-openbao.md`（OpenBao secret 管理）
- 関連 Runbook: `ops/runbooks/secret-rotation.md`（廃棄前に secret rotation を実施すること）
