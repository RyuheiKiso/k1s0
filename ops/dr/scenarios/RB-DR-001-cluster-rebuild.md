---
runbook_id: RB-DR-001
title: Disaster Recovery 発動（クラスタ全壊からの再構築、RTO 4h）
category: DR
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: ClusterTotalDisaster (External)
fmea_id: FMEA-008
estimated_recovery: RTO 4h / RPO 24h
last_updated: 2026-05-02
---

# RB-DR-001: Disaster Recovery 発動（クラスタ全壊からの再構築、RTO 4h）

本 Runbook は本番クラスタが全壊した（Control Plane 応答喪失、etcd 完全喪失、データセンタ全停止）場合の DR 発動手順を定める。NFR-A-REC-001（RTO 4h）、NFR-A-CONT-001、FMEA-008 に対応する。

`dr/scenarios/` 配下に置く理由は、本 Runbook が単純なインシデント対応ではなく「災害宣言から DR 環境への切替」という長時間・複合作業を扱い、実行には経営判断（Product Council 召集）が含まれるため。incident Runbook（〜1 時間で完了）とは性格を分離する。

## 1. 前提条件

- 実行者は `k1s0-cluster-admin` 権限を保持し、Cloud Provider（GCP / AWS / オンプレ Cloudflare）の管理コンソールにもアクセス可能なこと。
- 必要ツール: `kubectl` / `gcloud` または `aws` / `argocd` / `jq` / `cosign`（署名検証用）。
- 災害宣言の権限: 起案者または CTO が承認。協力者単独では発動不可。
- DR 候補リージョン / クラウドが事前定義されていること（`docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md` の DS-OPS-ENV-012）。
- MinIO クロスリージョンバックアップが直近 24h 以内に成功している（`kubectl cnpg backup list` で確認）。

## 2. 対象事象

- 外部 Mimir / 監視で `up{job="kubernetes-nodes"} == 0` がクラスタ全体で 30 分以上継続、または
- ArgoCD が全 Application を Degraded/Unknown と報告（`sum(argocd_app_info{health_status!="Healthy"}) / count(argocd_app_info{}) > 0.8`）、または
- `kubectl cluster-info --request-timeout=10s` が応答せず Control Plane VM が起動不能、または
- etcd データの完全喪失が確認された場合、または
- SEV1 が 3 時間経過しても復旧目処が立たない場合（経営判断）。

検知シグナル（外部監視前提）:

```promql
# クラスタ内 scrape が全停止
up{job="kubernetes-nodes"} == 0

# ArgoCD が全アプリケーションを Degraded/Unknown と報告
sum(argocd_app_info{health_status!="Healthy"}) / count(argocd_app_info{}) > 0.8
```

```bash
# 手動確認
kubectl cluster-info --request-timeout=10s
```

検知経路: PagerDuty の External HTTP チェック失敗 → Slack `#incident-sev1` → SRE オンコール + Engineering Manager + CTO 即時エスカレーション。

## 3. 初動手順（5 分以内）

```bash
# 障害範囲の確認（クラスタ全体 / 特定 AZ / Control Plane のみ）
kubectl get nodes --request-timeout=30s

# Cloud Provider コンソールで Node / VM の状態確認（GCP / AWS / Azure）
```

ステークホルダー第一報（5 分以内）:

- Slack `#status` に「調査中、DR 判定中」を投稿。
- 顧客向け Status Page を「調査中」に更新。
- CTO に電話、Product Council 召集準備。

DR 判定基準:

- Control Plane が 30 分以上応答なし、かつ回復見込みなし → **DR 発動**
- etcd データの完全喪失が確認された場合 → **DR 発動**
- 部分的障害で 1 時間以内に復旧見込み → DR 発動せず通常 SEV1 対応継続

```bash
# バックアップの最終確認時刻
aws s3 ls s3://k1s0-postgres-backup/ --endpoint-url http://<minio-external-lb>:9000 \
  --no-verify-ssl | tail -10
```

DR 発動が決定したら:

- Product Council 召集（CTO + 法務 + プラットフォーム責任者）。
- DR ターゲット環境を決定（別リージョン / 別クラウド / オンプレ）。
- 顧客向け Status Page を「DR 発動、RTO 4h」に更新。

## 4. 原因特定手順

DR 発動と並行して原因調査を進める（ポストモーテム用）。原因特定が DR 発動を遅らせてはならない。

よくある原因:

1. **etcd データ破損**: etcd の quorum 喪失や disk corruption。`etcdctl endpoint health` と `etcdctl snapshot status` で確認（旧クラスタが部分的に応答する場合）。
2. **Control Plane VM の同時障害**: Cloud Provider の AZ 障害でマルチ AZ でも同時に VM が停止。Cloud Provider の Incident Dashboard を確認。
3. **ネットワーク分断（Network Partition）**: VPC や NACLs の設定変更で Control Plane と Worker が分断。Cloud Provider のネットワークログを確認。FMEA-008 と一致。
4. **悪意のある kubectl コマンド（誤操作）**: `kubectl delete namespace --all` や `kubectl drain --all` 等。ArgoCD の audit ログと kubectl audit ログを確認。
5. **StorageClass / PV の一括削除**: PVC の reclaimPolicy が Delete の場合に全データが消える。StorageClass の reclaimPolicy が `Retain` であることを事後確認。

エスカレーション: 悪意のある操作が疑われる場合は `oncall/escalation.md` 経由で security-sre と法務に同時連絡。

## 5. 復旧手順

### Phase 1 — DR クラスタのブートストラップ（〜60 分）

```bash
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

## 6. 検証手順

復旧完了の判定基準（RTO 4h 内達成必須）:

- 全 Argo Application が `Healthy / Synced`（`argocd app list -o name | xargs -I{} argocd app wait {} --health --timeout 300`）。
- tier1 facade `/healthz` が 200（`curl https://api.k1s0.example.com/healthz` で外部疎通確認）。
- PostgreSQL Primary が応答（`SELECT pg_is_in_recovery()` が `f`）、`pg_database_size('k1s0')` が想定値の 95% 以上（バックアップが完全に展開されたか）。
- Kafka 全 broker が `Running`、`kafka_server_replicamanager_underreplicatedpartitions == 0`。
- MinIO の bucket 一覧が想定どおり（`mc ls` で audit / postgres-backup 等を確認）。
- Keycloak のログイン動作確認（管理者ログインで セッションが取得できる）。
- 外部監視（Statuscake / Pingdom 等）が DR LB IP で OK を返す。
- DNS 切替が浸透（`dig api.k1s0.example.com` が DR LB IP を返す、TTL 60s 設定済みなら 5 分以内）。

RTO 達成記録: 災害宣言時刻 と 検証完了時刻 を記録（4 時間以内なら NFR-A-REC-001 達成）。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-RB-DR-001.md`）。
- DR 訓練スケジュールの実施（四半期ごとに table-top exercise、半期に 1 回実機演習 — `ops/dr/drills/DR-drill-YYYY-Qn.md`）。
- RTO 4h / RPO 24h の達成可否をポストモーテムで記録し、未達の場合は改善 plan を策定。
- MinIO クロスリージョンレプリケーションの有効化を検討（RPO 短縮: 24h → 1h）。
- ArgoCD App of Apps の bootstrap 手順を `infra/k8s/bootstrap/README.md` に文書化。
- NFR-A-REC-001 / NFR-A-REC-002 の MTTR / MTBF ログを更新。
- DR 自動化の検討: `ops/dr/scripts/rebuild-cluster-from-scratch.sh` を整備し、Phase 1〜3 の手動ステップを縮減。

## 8. 関連 Runbook

- 関連設計書: `infra/k8s/bootstrap/`、`infra/data/cloudnativepg/cluster.yaml`、[`docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md)
- 関連 ADR: [ADR-DIR-001](../../../docs/02_構想設計/adr/ADR-DIR-001-monorepo.md)（モノレポ構造）
- 関連 NFR: [NFR-A-REC-001 / NFR-A-CONT-001](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 FMEA: [FMEA-008](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-DB-002-postgres-primary-failover.md`](../../runbooks/incidents/RB-DB-002-postgres-primary-failover.md) — 部分復旧で済む場合（DR 発動回避）
  - [`RB-SEC-002-cert-expiry.md`](../../runbooks/incidents/RB-SEC-002-cert-expiry.md) — DR 後に証明書が期限切れになっている場合
  - [`RB-BKP-001-backup-failure.md`](../../runbooks/incidents/RB-BKP-001-backup-failure.md) — DR の前提となるバックアップが失敗していた場合
- DR シナリオ:
  - [`pg-restore.md`](pg-restore.md) — 部分的 PostgreSQL リストア（DR 全体ではなく単一 cluster）
  - [`kafka-topic-restore.md`](kafka-topic-restore.md)
  - [`minio-tenant-restore.md`](minio-tenant-restore.md)
