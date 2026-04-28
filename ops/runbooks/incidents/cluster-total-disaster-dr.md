# クラスタ全壊 DR Runbook

> **alert_id**: tier1.infra.cluster.total-disaster
> **severity**: SEV1
> **owner**: sre-ops
> **estimated_mttr**: 240m (RTO 4h / RPO 24h)
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

**Mimir / Grafana** で以下を確認する（外部監視が生き残っている前提）。

PromQL（外部 Mimir または Uptime モニタリング）:

```promql
# クラスタ内 scrape が全停止（Prometheus/Mimir がスクレイプできない）
up{job="kubernetes-nodes"} == 0

# ArgoCD が全アプリケーションを Degraded/Unknown と報告
sum(argocd_app_info{health_status!="Healthy"}) / count(argocd_app_info{}) > 0.8
```

**手動確認**:

```bash
# kubectl が応答しない場合は kubeconfig の context を確認
kubectl cluster-info --request-timeout=10s
```

検知経路: PagerDuty の External HTTP チェック失敗 → Slack `#incident-sev1` → SRE オンコール + Engineering Manager 即時エスカレーション。

## 2. 初動 (Immediate Action, 〜15 分)

- [ ] 障害範囲の確認（クラスタ全体 / 特定 AZ / Control Plane のみ）

  ```bash
  # Cloud Provider コンソールで Node / VM の状態を確認（GCP / AWS / Azure）
  # k8s control plane の VM が起動しているか確認
  kubectl get nodes --request-timeout=30s
  ```

- [ ] ステークホルダーへの第一報（15 分以内）

  Slack `#status` チャンネルに「調査中」のステータスを投稿。顧客向け Status Page を「調査中」に更新。

- [ ] DR 判定: 以下の条件を満たす場合に DR 手順に移行する

  - Control Plane が 30 分以上応答なし、かつ回復見込みなし
  - または etcd データの完全喪失が確認された場合

- [ ] バックアップの最終確認時刻を確認する

  ```bash
  # CNPG backup リスト（MinIO から確認）
  aws s3 ls s3://k1s0-postgres-backup/ --endpoint-url http://<minio-external-lb>:9000 \
    --no-verify-ssl | tail -10
  ```

- [ ] DR クラスタのターゲット環境を決定する（別リージョン / 別クラウド / オンプレ）

## 3. 復旧 (Recovery, 〜60 分)

### Phase 1 — DR クラスタのブートストラップ（〜60 分）

```bash
# infra/k8s/bootstrap/ の手順に従い新クラスタを作成
# 1. k8s クラスタ作成（Cloud Provider CLI）
# 例: GKE の場合
gcloud container clusters create k1s0-dr \
  --region asia-northeast1 \
  --num-nodes 3 \
  --machine-type n2-standard-4

# 2. ArgoCD をブートストラップ
kubectl create namespace argocd
kubectl apply -n argocd -f \
  https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# 3. ArgoCD App of Apps でインフラ全体を再構築
kubectl apply -f infra/k8s/bootstrap/app-of-apps.yaml -n argocd
argocd app sync infra-root --prune
```

### Phase 2 — データリストア（〜90 分）

```bash
# CNPG クラスタを MinIO バックアップからリストア
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
        name: k1s0-pg-backup-latest
  externalClusters:
    - name: k1s0-pg-backup-latest
      barmanObjectStore:
        destinationPath: s3://k1s0-postgres-backup/
        endpointURL: http://<minio-new-endpoint>:9000
        s3Credentials:
          accessKeyId:
            name: k1s0-minio-credentials
            key: access-key
          secretAccessKey:
            name: k1s0-minio-credentials
            key: secret-key
EOF
kubectl get cluster k1s0-postgres -n cnpg-system -w
```

### Phase 3 — アプリケーション復旧（〜30 分）

```bash
# ArgoCD で tier1 → tier2 → tier3 の順に sync
argocd app sync tier1-services --prune
argocd app sync tier2-services --prune
argocd app sync tier3-services --prune

# SPIRE を最後に起動（CNPG が先に必要）
argocd app sync spire --prune
kubectl rollout status daemonset/spire-agent -n spire-system
```

### Phase 4 — トラフィック切替（〜15 分）

```bash
# DNS を DR クラスタの Load Balancer IP に向ける
# Cloudflare の場合
curl -X PUT "https://api.cloudflare.com/client/v4/zones/<zone-id>/dns_records/<record-id>" \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  -H "Content-Type: application/json" \
  --data '{"type":"A","name":"api.k1s0.example.com","content":"<dr-lb-ip>","ttl":60}'
```

## 4. 原因調査 (Root Cause Analysis)

**よくある原因**:

1. **etcd データ破損**: etcd の quorum 喪失や disk corruption。`etcdctl endpoint health` と `etcdctl snapshot status` で確認。
2. **Control Plane VM の同時障害**: Cloud Provider の AZ 障害でマルチ AZ でも同時に VM が停止。Cloud Provider の Incident Dashboard を確認。
3. **ネットワーク分断（Network Partition）**: VPC や NACLs の設定変更で Control Plane と Worker が分断。Cloud Provider のネットワークログを確認。
4. **悪意のある kubectl コマンド（誤操作）**: `kubectl delete namespace --all` や `kubectl drain --all` 等。ArgoCD の audit ログと kubectl audit ログを確認。
5. **StorageClass / PV の一括削除**: PVC の reclaimPolicy が Delete の場合に全データが消える。StorageClass の reclaimPolicy が `Retain` であることを事後確認。

## 5. 事後処理 (Post-incident)

- [ ] ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-cluster-total-disaster.md`）
- [ ] DR 訓練スケジュールの実施（四半期ごとに table-top exercise）
- [ ] RTO 4h / RPO 24h の達成可否をポストモーテムで記録し、未達の場合は改善 plan を策定
- [ ] MinIO クロスリージョンレプリケーションの有効化を検討（RPO 短縮）
- [ ] ArgoCD App of Apps の bootstrap 手順を `infra/k8s/bootstrap/README.md` に文書化
- [ ] NFR-A-REC-002 の MTTR / MTBF ログを更新

## 関連

- 関連設計書: `infra/k8s/bootstrap/`、`infra/data/cloudnativepg/cluster.yaml`
- 関連 ADR: `docs/02_構想設計/adr/ADR-DIR-001`（モノレポ構造）
- 関連 Runbook: `ops/runbooks/incidents/postgresql-primary-down.md`、`ops/runbooks/incidents/mtls-cert-expiry.md`
