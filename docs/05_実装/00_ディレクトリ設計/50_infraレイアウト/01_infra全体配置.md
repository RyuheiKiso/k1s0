# 01. infra 全体配置

本ファイルは `infra/` 配下の全体構成を確定する。旧 DS-SW-COMP-120 における `src/tier1/infra/` を廃止し、ルート `infra/` に昇格した状態を前提とする（ADR-DIR-002）。

## `infra/` の役割

`infra/` はクラスタ素構成を記述する場所である。以下 3 層のうち、最下層の「素構成」を担当する。

- **infra**: クラスタ素構成（namespace / CRD / Helm values）— GitOps が取り込む土台
- **deploy**: GitOps 配信定義（ArgoCD Application / Kustomize overlay）
- **ops**: 運用領域（Runbook / Chaos / DR / Oncall）

`infra/` に含まれる YAML は ArgoCD App-of-Apps パターンで `deploy/apps/` から参照され、各環境にデプロイされる。`infra/` 自体は「何をデプロイするか」を記述し、「どこにどう配信するか」は `deploy/` が担う。この分離により、Dev 環境と Prod 環境でコア設定を共有しつつ、環境差分を overlay で吸収できる。

## レイアウト

```
infra/
├── README.md
├── k8s/                    # Kubernetes ブートストラップ
│   ├── bootstrap/          # cluster bootstrap（CNI / CoreDNS / cert-manager 初期化）
│   ├── namespaces/         # namespace 定義とラベル付け
│   ├── networking/         # MetalLB / NetworkPolicy / gatewayclasses
│   └── storage/            # Longhorn StorageClass / CSI
├── mesh/                   # サービスメッシュ
│   ├── istio-ambient/      # Istio Ambient Mesh（ztunnel / waypoint）
│   └── envoy-gateway/      # Envoy Gateway（ingress）
├── dapr/                   # Dapr 基盤（旧 src/tier1/infra/dapr から移設）
│   ├── control-plane/      # dapr operator / placement / sentry / injector
│   └── components/         # Dapr Component CRD（state / pubsub / secrets / binding）
│       ├── state/
│       ├── pubsub/
│       ├── secrets/
│       ├── binding/
│       └── configuration/
├── data/                   # データ層基盤
│   ├── cloudnativepg/      # CloudNativePG Operator + Cluster CRD
│   ├── kafka/              # Strimzi Kafka Operator + Kafka CRD
│   ├── valkey/             # Valkey (Redis fork)
│   └── minio/              # MinIO Tenant CRD
├── security/               # セキュリティ基盤
│   ├── keycloak/           # Keycloak realm / client 定義
│   ├── openbao/            # OpenBao（Vault fork）
│   ├── spire/              # SPIFFE/SPIRE server + agent
│   ├── cert-manager/       # cert-manager ClusterIssuer
│   └── kyverno/            # Kyverno ClusterPolicy
├── observability/          # 観測性基盤
│   ├── otel-collector/     # OpenTelemetry Collector
│   ├── loki/               # Loki（Log）
│   ├── tempo/              # Tempo（Trace）
│   ├── mimir/              # Mimir（Metrics）
│   ├── grafana/            # Grafana dashboards
│   └── pyroscope/          # Pyroscope（Continuous Profiling）
├── feature-management/     # フィーチャーフラグ基盤
│   └── flagd/
├── scaling/                # オートスケーリング
│   └── keda/
└── environments/           # 環境差分パッチ
    ├── dev/
    ├── staging/
    └── prod/
```

## 依存方向

- `infra/` は `src/` を参照しない（素構成は実装に先行する）
- `deploy/` は `infra/` を Kustomize base / Helm values 経由で参照する
- `ops/` は `infra/` の CRD 定義を前提とした Runbook を記述する

## 各サブディレクトリの責務境界

### k8s/

Kubernetes 自体の最小構成を担う。CNI（Cilium or Calico）、CoreDNS、cert-manager CRD 初期化、namespace 作成、MetalLB / gatewayclass、Longhorn の StorageClass 定義がここに集まる。これらは全ての層（mesh / dapr / data / security / observability）が依存する最下位基盤。

### mesh/

Service mesh 層。k1s0 は Istio Ambient Mesh を採用（NFR-A-SEC-\* と NFR-B-PERF-\* の両立を sidecar-less で達成）。mesh/ には ztunnel DaemonSet 定義と waypoint proxy の HelmChart を配置。

### dapr/

Dapr Control Plane（operator / placement / sentry / injector / scheduler）と、Dapr Component CRD。Component は `state/` `pubsub/` `secrets/` `binding/` `configuration/` の 5 カテゴリに分ける。Component は tier1 公開 API（State/PubSub/Secrets/Binding）の backing store 選択を担う。

### data/

データ層の永続化基盤。CloudNativePG（PostgreSQL）、Strimzi Kafka、Valkey（Redis fork）、MinIO の 4 Operator を配置する。Operator / Cluster CRD は k8s/namespaces/ で作成した `k1s0-data` namespace に配置。

### security/

認証認可・機密管理・PKI・ポリシー。Keycloak は OIDC IdP、OpenBao（Vault fork）は secret 管理、SPIRE は mTLS workload identity、cert-manager は TLS 証明書発行、Kyverno は admission policy。

### observability/

LGTM スタック（Loki / Grafana / Tempo / Mimir）+ Pyroscope + OpenTelemetry Collector。log / trace / metrics / profile の 4 signals を一元管理。NFR-C-NOP-\* の観測性要件を満たす。

### feature-management/ / scaling/

flagd はフィーチャーフラグ（Feature Management、OpenFeature 準拠）。KEDA は event-driven autoscaling（Kafka lag や OpenTelemetry metrics を watch し HPA を生成）。

### environments/

dev / staging / prod の環境差分を Kustomize overlay で記述する。具体例: dev は CloudNativePG が instances: 1、prod は instances: 3（高可用性）など。各環境 overlay が本 infra/ の base を patch する。

## 導入段階

| 適用段階 | 対象 |
|---|---|
| リリース時点 | 構造のみ（ディレクトリと README） |
| リリース時点 | k8s/bootstrap + k8s/namespaces（最小 k3s 起動） |
| リリース時点 | mesh / dapr / observability の最小導入、data 層 1 Operator |
| リリース時点 | security 全層、data 層全 Operator、feature-management、scaling |
| 採用後の運用拡大時 | environments/ 全環境、DR / multi-region 対応 |

## 対応 IMP-DIR ID

- IMP-DIR-INFRA-071（infra 全体配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-DIR-002（infra 分離）
- DS-SW-COMP-120（改訂前）の移行先
- NFR-A-AVL-\* / NFR-B-PERF-\* / NFR-C-NOP-\* / NFR-E-SEC-\*
