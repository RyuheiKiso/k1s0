# infra — k8s クラスタ素構成（Operator / data backend / mesh / observability / security）

ADR-DIR-002（infra 分離）に従い、**マニフェスト形式のインフラ宣言** のみを集約する。
GitOps 配信定義（ApplicationSet 等）は [`deploy/`](../deploy/) に分離され、本ディレクトリは
「クラスタに何が乗っているか」の宣言だけを持つ。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/`](../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/)。

## namespace 規約（spec 正典）

全 namespace は `k1s0-` プレフィックスを必須とする。tier 層 `k1s0-tier1/2/3`、機能層
`k1s0-{data,dapr,mesh,security,obs,platform-tools}`。サードパーティ operator が namespace
を hardcode する `istio-system` / `envoy-gateway-system` / `capi-system` のみ例外として残す。

| namespace | 配置されるコンポーネント |
|---|---|
| `k1s0-tier1` | tier1 facade + Dapr Component / Subscription（k1s0-tier1 ns 内 CR）|
| `k1s0-tier2` | tier2 ドメインサービス（DDD bounded context 単位）|
| `k1s0-tier3` | tier3 BFF / Web / Native のサーバ側 |
| `k1s0-data` | CNPG (3 Cluster) / Kafka / MinIO (3 Tenant) / Valkey (Cluster mode) |
| `k1s0-dapr` | Dapr operator / placement / sentry / injector / scheduler |
| `k1s0-mesh` | Envoy Gateway / Istio user-facing CR（waypoint / authz）|
| `k1s0-security` | Keycloak / OpenBao / SPIRE / cert-manager / Kyverno |
| `k1s0-obs` | LGTM (Loki / Grafana / Tempo / Mimir) + Pyroscope + OTel + Alertmanager |
| `k1s0-platform-tools` | Argo CD / Backstage / flagd / KEDA / MetalLB / 内部 registry / Gitea |

## 配置

```text
infra/
├── k8s/
│   ├── bootstrap/                          # Cluster API + KubeadmControlPlane / kubeadm-init / cilium / coredns / cert-manager-crd
│   ├── namespaces/                         # k1s0-* 9 ns + サードパーティ operator ns + PSS label + Istio Ambient mode label
│   ├── networking/                         # Cilium eBPF (kube-proxy replacement + Hubble) + MetalLB + GatewayClass + 全 ns default-deny NetworkPolicy
│   └── storage/                            # Longhorn + 2 StorageClass（longhorn-retain / longhorn-delete）
├── mesh/
│   ├── istio-ambient/                      # operator / ztunnel / waypoint / authz / peerauthentication（STRICT mTLS）
│   ├── envoy-gateway/                      # Gateway API 標準準拠の南北 ingress
│   └── envoy-grpcweb/                      # gRPC-Web プロキシ（拡張領域、tier3-web→tier1 gRPC ブリッジ）
├── dapr/
│   ├── control-plane/                      # crd / operator / placement / sentry / injector / scheduler（HA 3）
│   ├── components/                         # Dapr Component CRD（state / pubsub / secrets / binding / configuration / workflow）
│   └── subscriptions/                      # Dapr Subscription CRD（audit-pii / feature 配信）
├── data/                                   # ADR-DATA-001/002/003/004
│   ├── cloudnativepg/                      # 3 Cluster (tier1-state / tier2-domain / audit) + backup + monitoring
│   ├── kafka/                              # Strimzi KRaft 3 broker + topics + users (mTLS ACL) + mirror-maker
│   ├── minio/                              # 3 Tenant (tier1-binding / audit-archive / observability) + bucket lifecycle
│   └── valkey/                             # Cluster mode 6 node (master 3 + replica 3)
├── security/                               # ADR-SEC-001/002/003 / ADR-POL-001
│   ├── cert-manager/                       # HA + Let's Encrypt prod/staging + 内部 CA + csi-driver + approvers
│   ├── keycloak/                           # HA 3 + 2 realm 分離 (k1s0-internal / k1s0-tenant) + clients
│   ├── openbao/                            # HA 3 + Raft + auth-methods (kubernetes / spire) + secret-engines (kv-v2 / database / pki) + policies
│   ├── spire/                              # server HA 3 + agent DaemonSet + ClusterSPIFFEID + csi-driver、trustDomain=k1s0.internal
│   └── kyverno/                            # admission + 5 必須 ClusterPolicy + ADR-POL-002 drift 防止
├── observability/                          # ADR-OBS-001/002（LGTM + Pyroscope + Alertmanager）
│   ├── grafana/                            # HA 2 + Keycloak OIDC + datasources / dashboards CR
│   ├── loki/                               # SimpleScalable + MinIO observability Tenant
│   ├── tempo/                              # distributed + MinIO + memcached
│   ├── mimir/                              # distributed + MinIO + zoneAwareReplication
│   ├── pyroscope/                          # micro-services + MinIO
│   ├── otel-collector/                     # agent DaemonSet + gateway HA 3 + processors (k8sattributes / pii-redact / sampling)
│   ├── alertmanager/                       # HA 3 + PagerDuty / Slack / mailto receivers
│   └── alerts/                             # PrometheusRule (SLO + tier1/2/3 alert)
├── scaling/keda/                           # operator HA 2 + metrics-apiserver + admission webhook
├── feature-management/flagd/               # HA 3 + ConfigMap baseline flag + ServiceMonitor
├── registry/local/                         # 拡張領域: 自前 OCI image registry（k1s0-platform-tools ns）
├── gitops/local-stack/                     # 拡張領域: ローカル GitOps シミュレーション（Gitea + Argo CD repo secret）
└── environments/                           # 環境別 overlay（IMP-DIR-INFRA-078）
    ├── dev/                                # CNPG 3→1 / Kafka 1 broker / Valkey standalone / 軽量 LGTM (7d/3d) / Let's Encrypt staging
    ├── staging/                            # CNPG 3→2 / Kafka 3 broker (storage 50Gi) / prod 同等 HA
    └── prod/                               # 空（base がそのまま prod）
```

「拡張領域」と表記したサブツリー（`mesh/envoy-grpcweb/` / `registry/local/` / `gitops/local-stack/`）は、現時点で `IMP-DIR-INFRA-*` には未採番だが運用上必要なため infra 配下に残してある。後続で ADR / IMP-DIR-INFRA に正式採番する予定。

## マニフェストの編成原則

- **Helm values と Kustomize patch を分離**: 各 component 配下に `values.yaml`（Helm 用）と
  CR 宣言を併置。環境別差分は `infra/environments/<env>/values/<component>/values.yaml` および
  `patches/` で overlay。
- **Operator と CR の Sync Wave 分離**: Operator は Wave -10、CR 宣言は Wave -5、data backend は Wave 0。
  Argo CD Sync Wave で起動順序を物理的に強制（各サブの `*/kustomization.yaml` の `commonAnnotations`）。
- **AGPL の取り扱い**: MinIO ほか AGPL 依存 OSS は infra 内部でのみ稼働し、ネットワーク越しに改変版を
  ユーザに提供しない。法務判定は [`docs/02_構想設計/05_法務とコンプライアンス/`](../docs/02_構想設計/05_法務とコンプライアンス/)。
- **3 環境統一**: dev / staging / prod は同じ `infra/` base を参照し、replica 数 / リソース上限のみ
  patch で差替（DS-OPS-ENV-007 の本番同等性）。

## 採用検討段階のローカル再現

`tools/local-stack/up.sh` で k1s0-* 9 namespace + 主要 component の最小構成を kind クラスタ上に立ち上げられる
（[`tools/local-stack/`](../tools/local-stack/) 参照）。リリース時点最小は `k1s0-tier1` `k1s0-data`
`k1s0-dapr` `k1s0-mesh` `k1s0-obs` の 5 ns（`02_k8sブートストラップ.md`）。

## 関連設計

- [ADR-DIR-002](../docs/02_構想設計/adr/ADR-DIR-002-infra-separation.md) — infra 分離
- [ADR-MIG-002](../docs/02_構想設計/adr/ADR-MIG-002-istio-ambient-migration.md) — Ambient Mesh
- [ADR-DATA-001〜004](../docs/02_構想設計/adr/) — data backend 選定
- [ADR-SEC-001/002/003](../docs/02_構想設計/adr/) — Keycloak / OpenBao / SPIRE
- [ADR-OBS-001/002](../docs/02_構想設計/adr/) — Grafana LGTM
- [ADR-POL-001](../docs/02_構想設計/adr/ADR-POL-001-kyverno-dual-owner.md) — Kyverno 二重オーナー
- [ADR-POL-002](../docs/02_構想設計/adr/ADR-POL-002-local-stack-single-source-of-truth.md) — drift 防止
