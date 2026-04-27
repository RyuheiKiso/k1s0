# infra — k8s クラスタ素構成（Operator / data backend / mesh / observability / security）

ADR-DIR-002（infra 分離）に従い、**マニフェスト形式のインフラ宣言** のみを集約する。
GitOps 配信定義（ApplicationSet 等）は [`deploy/`](../deploy/) に分離され、本ディレクトリは
「クラスタに何が乗っているか」の宣言だけを持つ。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/`](../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/)。

## 配置

```text
infra/
├── k8s/
│   ├── bootstrap/                          # Cluster API + KubeadmControlPlane HA 3 / kubeadm-init 代替
│   ├── namespaces/                         # 17 layer namespace + Pod Security Standards label + Istio Ambient mode label
│   ├── networking/                         # Calico VXLAN + MetalLB（IPAddressPool は環境別 overlay）
│   └── storage/                            # 4 種 StorageClass（default / high-iops / shared / backup）
├── mesh/
│   ├── istio-ambient/                      # CNI + ztunnel + istiod HA 3、PeerAuthentication STRICT
│   └── envoy-gateway/                      # Gateway API 標準準拠の南北 ingress
├── dapr/
│   ├── control-plane/                      # operator / placement / sentry / sidecar-injector / scheduler（HA 3）
│   ├── components/                         # Dapr Component CRD（state / pubsub / secrets / binding / configuration）
│   └── subscriptions/                      # Dapr Subscription CRD（audit-pii / feature 配信）
├── data/                                   # ADR-DATA-001/002/003/004
│   ├── cloudnativepg/                      # 3 instance HA + WAL アーカイブ + PodMonitor
│   ├── kafka/                              # Strimzi KRaft 3 broker + TLS mTLS + Cruise Control
│   ├── minio/                              # distributed mode 4 replica + erasure coding
│   └── valkey/                             # replication + Sentinel + 認証有効
├── security/                               # ADR-SEC-001/002/003 / ADR-POL-001
│   ├── cert-manager/                       # HA 3 + Let's Encrypt + 内部 CA ClusterIssuer
│   ├── keycloak/                           # HA 3 + Infinispan + 外部 CNPG + production mode
│   ├── openbao/                            # HA 3 + Raft integrated storage + Vault Agent Injector
│   ├── spire/                              # server HA 3 + 外部 CNPG + cert-manager upstream + CSI driver
│   └── kyverno/                            # admission HA 3 + 4 baseline ClusterPolicy
├── observability/                          # ADR-OBS-001/002（LGTM スタック）
│   ├── grafana/                            # HA 2 + Keycloak OIDC + sidecar pickup
│   ├── loki/                               # SimpleScalable + S3
│   ├── tempo/                              # distributed + S3 + memcached
│   ├── mimir/                              # distributed + S3 + zoneAwareReplication
│   ├── pyroscope/                          # micro-services + S3
│   └── otel-collector/                     # agent DaemonSet + gateway Deployment 3 replica
├── scaling/keda/                           # operator HA 2 + metrics-apiserver + admission webhook
├── feature-management/flagd/               # HA 3 + ConfigMap baseline flag + ServiceMonitor
└── environments/                           # 環境別 overlay（IMP-DIR-INFRA-078）
    ├── dev/                                # CNPG 3→1 / Kafka 100Gi→5Gi / 軽量 LGTM
    ├── staging/                            # CNPG 3→2 / Kafka 100Gi→50Gi
    └── prod/                               # 空（base がそのまま prod）
```

## マニフェストの編成原則

- **Helm values と Kustomize patch を分離**: 各 component 配下に `values.yaml`（Helm 用）と
  `<component>-cluster.yaml`（CR 宣言）を併置。環境別差分は `infra/environments/<env>/values/<component>/values.yaml` で overlay。
- **Operator と CR の Sync Wave 分離**: Operator は Wave -10、CR 宣言は Wave -5、data backend は Wave 0。
  Argo CD Sync Wave で起動順序を物理的に強制（[`deploy/apps/`](../deploy/apps/) 参照）。
- **AGPL の取り扱い**: MinIO ほか AGPL 依存 OSS は infra 内部でのみ稼働し、ネットワーク越しに改変版を
  ユーザに提供しない。法務判定は [`docs/02_構想設計/05_法務とコンプライアンス/`](../docs/02_構想設計/05_法務とコンプライアンス/)。

## 採用検討段階のローカル再現

`tools/local-stack/up.sh` で 17 layer namespace + 主要 component の最小構成を kind クラスタ上に立ち上げられる
（[`tools/local-stack/`](../tools/local-stack/) 参照）。

## 関連設計

- [ADR-DIR-002](../docs/02_構想設計/adr/ADR-DIR-002-infra-separation.md) — infra 分離
- [ADR-MIG-002](../docs/02_構想設計/adr/ADR-MIG-002-istio-ambient-migration.md) — Ambient Mesh
- [ADR-DATA-001〜004](../docs/02_構想設計/adr/) — data backend 選定
- [ADR-SEC-001/002/003](../docs/02_構想設計/adr/) — Keycloak / OpenBao / SPIRE
- [ADR-OBS-001/002](../docs/02_構想設計/adr/) — Grafana LGTM
