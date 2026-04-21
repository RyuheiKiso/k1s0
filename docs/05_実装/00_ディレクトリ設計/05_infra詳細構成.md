# 05. infra 詳細構成

本ファイルは `src/tier1/infra/` 配下の詳細ファイル配置を方式として固定化する。上流は概要設計 [DS-SW-COMP-120（tier1 のトップレベル構成）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)・[55_運用ライフサイクル方式設計/03_環境構成管理方式](../../04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md)・[70_開発者体験方式設計/01_CI_CD方式](../../04_概要設計/70_開発者体験方式設計/01_CI_CD方式.md)で、本ファイルは `infra/` 配下を **`delivery/`（デプロイ成果物）・`runtime/`（Pod 実行時の Dapr 設定）・`governance/`（統制）・`platform/`（基盤プラットフォーム）の 4 カテゴリ**に分類したうえで、Helm chart・Kustomize overlay・Dapr Components・Argo Rollouts・Kyverno ポリシー・Operators・バックエンド CR のファイル配置を確定させる。

## 本ファイルの位置付け

概要設計は `src/tier1/infra/` を「Kubernetes manifests / Dapr Components / Helm charts を配置する」とだけ宣言し、詳細は未確定だった。本ファイルは (a) `infra/` 配下の 4 カテゴリ分類、(b) Helm chart の命名と構造、(c) Kustomize overlay の環境分離戦略、(d) Dapr Component YAML の配置、(e) Argo Rollouts / Kyverno / Operators のマニフェスト配置、(f) バックエンド CR の配置を実装視点で確定させる。

4 カテゴリ分離の根拠は 3 点ある。第 1 に、`delivery/` と `runtime/` は変更頻度・変更主体が大きく異なる。`delivery/` は PR 毎にアプリの変更と同期して更新され、Go/Rust エンジニアが触る。`runtime/` は Dapr Building Block 設定で、プラットフォームエンジニアが触り、変更頻度は低い。同じ階層に混在させると CODEOWNERS の path filter が煩雑になる。第 2 に、`governance/`（Kyverno / Namespace）は security-team・infra-team の両承認を必須とする統制領域で、他カテゴリと承認ルールが根本的に異なる。物理的に分離することで `CODEOWNERS` の記述が明瞭になる。第 3 に、`platform/` は Operator と Backend CR という「基盤を動かす領域」で、Operator（実行プロセス）と Backend（ワークロード）の対がある。Operator 毎にサブグルーピング（messaging / storage / security / delivery）することで、Phase 2 で新規基盤（例: Redpanda）を追加する際の配置決定が機械的に行える。

infra/ 配下の配置は GitOps の根幹である。[../../04_概要設計/70_開発者体験方式設計/01_CI_CD方式.md](../../04_概要設計/70_開発者体験方式設計/01_CI_CD方式.md) で「GitOps リポジトリ（`k1s0-gitops`）」と本リポジトリ（`k1s0`）の役割分離が定義されているが、本章は「本リポジトリ側に何を置くか」を扱う。本リポジトリの `infra/` には **Helm chart とテンプレートの真実**を配置し、GitOps リポジトリには **環境ごとの values と Application 定義**を配置する。この分離により、Helm chart の変更は本リポジトリのレビューで検証され、values の変更は GitOps リポジトリのレビューで検証される（関心の分離）。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-IMPL-DIR-121` 〜 `DS-IMPL-DIR-160` の 40 件である。

## infra/ 全体構造

`src/tier1/infra/` の完全な構造は以下のとおり。4 カテゴリ（`delivery/` / `runtime/` / `governance/` / `platform/`）に分類し、`platform/` 配下は operators と backends を機能グループ別（messaging / storage / security / delivery）にサブグルーピングする。

```
src/tier1/infra/
├── README.md                               # infra 概要と GitOps リポジトリとの関係
├── delivery/                               # デプロイ成果物（PR 同期で更新、tier1-go/rust team 主体）
│   ├── helm/                               # Helm Chart の真実
│   │   ├── tier1-facade/                   # ファサード 3 Pod 共通 Chart
│   │   │   ├── Chart.yaml
│   │   │   ├── values.yaml                 # デフォルト値（プレーン / dev 向け）
│   │   │   ├── values.schema.json          # JSON Schema でキー検証
│   │   │   ├── templates/
│   │   │   │   ├── _helpers.tpl
│   │   │   │   ├── deployment.yaml
│   │   │   │   ├── service.yaml
│   │   │   │   ├── configmap.yaml
│   │   │   │   ├── serviceaccount.yaml
│   │   │   │   ├── networkpolicy.yaml
│   │   │   │   ├── pdb.yaml                # PodDisruptionBudget
│   │   │   │   ├── hpa.yaml                # HorizontalPodAutoscaler
│   │   │   │   └── servicemonitor.yaml     # Prometheus Operator
│   │   │   └── tests/
│   │   │       └── test-connection.yaml
│   │   ├── tier1-rust/                     # 自作 Rust 3 Pod 共通 Chart
│   │   │   ├── Chart.yaml
│   │   │   ├── values.yaml
│   │   │   ├── values.schema.json
│   │   │   └── templates/
│   │   │       ├── _helpers.tpl
│   │   │       ├── deployment.yaml
│   │   │       ├── service.yaml
│   │   │       ├── configmap.yaml
│   │   │       ├── serviceaccount.yaml
│   │   │       ├── networkpolicy.yaml
│   │   │       ├── pdb.yaml
│   │   │       ├── hpa.yaml
│   │   │       └── servicemonitor.yaml
│   │   └── tier1-umbrella/                 # 6 Pod を束ねる umbrella chart
│   │       ├── Chart.yaml                  # dependencies で tier1-facade / tier1-rust を include
│   │       ├── values.yaml                 # 全 Pod の初期 values
│   │       └── templates/
│   │           └── NOTES.txt
│   ├── kustomize/                          # Kustomize overlay（Phase 1b 以降の追加加工）
│   │   ├── base/
│   │   │   ├── kustomization.yaml
│   │   │   └── helm-release.yaml           # Helm を Kustomize から呼ぶ
│   │   └── overlays/
│   │       ├── dev/
│   │       │   ├── kustomization.yaml
│   │       │   └── patches/
│   │       ├── staging/
│   │       └── prod/
│   └── rollouts/                           # Argo Rollouts（Phase 2 以降）
│       ├── README.md                       # Phase 1a は README のみ
│       ├── rollout-tier1-facade.yaml       # Phase 2 で追加
│       ├── rollout-tier1-rust.yaml         # Phase 2 で追加
│       └── analysis-templates/
│           ├── error-rate.yaml
│           ├── p99-latency.yaml
│           └── request-volume.yaml
├── runtime/                                # Pod 実行時の Dapr 設定（infra-team 主体、変更頻度低）
│   ├── components/                         # Dapr Component CRD（Building Block 別）
│   │   ├── state/
│   │   │   ├── valkey-state.yaml           # State Store: Valkey（Phase 2）
│   │   │   └── memory-state.yaml           # ローカル開発用 in-memory（Phase 1a）
│   │   ├── pubsub/
│   │   │   └── kafka-pubsub.yaml           # Phase 2
│   │   ├── secrets/
│   │   │   └── openbao-secrets.yaml        # Phase 1b
│   │   ├── bindings/
│   │   │   ├── http-binding.yaml           # Phase 1b
│   │   │   ├── smtp-binding.yaml           # Phase 1c
│   │   │   └── s3-binding.yaml             # Phase 1c
│   │   └── workflows/
│   │       └── dapr-workflow.yaml          # Phase 1b
│   ├── configuration/                      # Dapr Configuration
│   │   ├── configuration.yaml              # Dapr 全体設定（tracing / metric / mtls / AC）
│   │   └── resiliency.yaml                 # resiliency ポリシー
│   └── subscriptions/                      # Dapr Subscription
│       ├── audit-subscription.yaml         # Phase 1b
│       └── pii-detected-subscription.yaml  # Phase 1c
├── governance/                             # 統制（security-team + infra-team 両承認）
│   ├── policies/                           # Kyverno ClusterPolicy
│   │   ├── README.md                       # audit → enforce 遷移計画
│   │   ├── require-harbor-registry.yaml
│   │   ├── deny-latest-tag.yaml
│   │   ├── require-cosign-signature.yaml   # Phase 2
│   │   ├── require-podsecurity-restricted.yaml
│   │   ├── require-networkpolicy.yaml
│   │   ├── require-resources-limits.yaml
│   │   └── require-non-root.yaml
│   └── namespaces/                         # Namespace 定義（PodSecurityStandard 付与）
│       ├── k1s0-tier1.yaml
│       ├── k1s0-dapr.yaml
│       ├── k1s0-kafka.yaml
│       ├── k1s0-postgres.yaml
│       ├── k1s0-openbao.yaml
│       └── k1s0-monitoring.yaml
└── platform/                               # 基盤プラットフォーム（infra-team 主体）
    ├── operators/                          # Operator インストール用マニフェスト（実行プロセス）
    │   ├── messaging/
    │   │   ├── dapr/
    │   │   │   └── dapr-operator.yaml
    │   │   └── strimzi/
    │   │       └── strimzi-cluster-operator.yaml
    │   ├── storage/
    │   │   ├── cnpg/                       # CloudNativePG
    │   │   │   └── cnpg-operator.yaml
    │   │   └── minio/
    │   │       └── minio-operator.yaml     # Phase 1c
    │   ├── security/
    │   │   ├── openbao/
    │   │   │   └── openbao-operator.yaml
    │   │   └── external-secrets/           # Phase 1b
    │   │       └── external-secrets.yaml
    │   └── delivery/
    │       ├── argo-cd/
    │       │   └── argocd.yaml
    │       ├── argo-rollouts/              # Phase 2
    │       │   └── argo-rollouts.yaml
    │       ├── kyverno/
    │       │   └── kyverno.yaml
    │       └── istio/                      # Istio Ambient（Phase 2）
    │           ├── istiod.yaml
    │           ├── ztunnel.yaml
    │           └── waypoint.yaml
    └── backends/                           # バックエンド OSS の Cluster Resource（ワークロード）
        ├── messaging/
        │   └── kafka/                      # Phase 2
        │       ├── README.md
        │       ├── kafka-cluster.yaml      # Strimzi Kafka CR
        │       ├── topics/
        │       │   ├── audit-events.yaml
        │       │   └── pii-detected.yaml
        │       └── users/
        │           └── tier1-users.yaml
        ├── storage/
        │   ├── postgres/
        │   │   ├── audit-cluster.yaml      # CNPG Cluster CR（Phase 1b）
        │   │   └── scheduled-backups/
        │   │       └── audit-backup.yaml
        │   ├── valkey/                     # Phase 2
        │   │   ├── README.md
        │   │   └── valkey-cluster.yaml
        │   └── minio/                      # Phase 1c
        │       └── minio-tenant.yaml
        └── security/
            └── openbao/
                ├── openbao-cluster.yaml    # Phase 1b
                └── auth-k8s.yaml           # Kubernetes Auth 有効化
```

## delivery/helm/ — Helm Chart

### DS-IMPL-DIR-121 Helm Chart 3 枚体制

`src/tier1/infra/delivery/helm/` には 3 つの Chart を配置する。`tier1-facade/` は Go 3 Pod（STATE / SECRET / WORKFLOW）を束ね、`tier1-rust/` は Rust 3 Pod（AUDIT / DECISION / PII）を束ね、`tier1-umbrella/` は両者を dependencies として include する。3 枚体制にする理由は以下。

1. facade と rust は Dapr sidecar の有無（DS-SW-COMP-053）が異なり、Chart template が根本的に異なる
2. facade と rust は resource 要件（CPU / memory 割当、HPA threshold）が異なる
3. tier1-umbrella 経由で「tier1 を丸ごとデプロイ」を 1 コマンドで実現（`helm install k1s0 ./delivery/helm/tier1-umbrella`）
4. Phase 2 の tier2 / tier3 Chart 追加時も umbrella パターンを再利用できる（追加 Chart は同 `delivery/helm/` 配下に追加するだけ）

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-SW-COMP-053、DS-SW-COMP-120。

### DS-IMPL-DIR-122 Chart.yaml の必須フィールド

各 Chart の `Chart.yaml` には以下を必須とする。

```yaml
apiVersion: v2
name: tier1-facade
description: k1s0 tier1 facade layer (Go + Dapr) 3 pods
type: application
version: 0.1.0                    # Chart 本体のバージョン
appVersion: "0.1.0"               # 中の app のバージョン
kubeVersion: ">=1.28.0"
maintainers:
  - name: k1s0 tier1 team
    email: tier1@k1s0.internal
dependencies: []                  # tier1-umbrella は facade/rust を宣言
```

`version` は Chart 本体の変更履歴（semver）、`appVersion` は中の app（tier1 Pod）のバージョン。両者を独立に管理することで、Chart template のリファクタリング（version bump）を app リリース（appVersion bump）と分離できる。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-DEVX-CICD-\*。

### DS-IMPL-DIR-123 values.schema.json の必須化

各 Chart は `values.schema.json` を配置し、`values.yaml` のキー名・型・範囲を JSON Schema で検証する。`helm install --strict` で schema 違反を検出できる（values.yaml に typo があった場合、実際に apply される前に fail させる）。schema は values の全トップレベルキーを網羅し、追加プロパティは `additionalProperties: false` で拒否する。

例: `tier1-facade/values.schema.json` の一部

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "additionalProperties": false,
  "required": ["replicas", "image", "resources"],
  "properties": {
    "replicas": { "type": "integer", "minimum": 1, "maximum": 10 },
    "image": {
      "type": "object",
      "required": ["repository", "tag"],
      "properties": {
        "repository": { "type": "string", "pattern": "^harbor\\.k1s0\\.internal/" },
        "tag": { "type": "string", "pattern": "^v\\d+\\.\\d+\\.\\d+-[a-f0-9]{7}$" }
      }
    },
    "resources": { "$ref": "#/$defs/resources" }
  }
}
```

`image.repository` は Harbor プレフィックス必須、`image.tag` は semver + git hash 形式必須で、それ以外は schema 違反で拒否する（Kyverno 側とも二重防御）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-SUP-\*。**上流**: DS-SW-COMP-137、DS-DEVX-CICD-\*。

### DS-IMPL-DIR-124 templates/ の必須 YAML 7 種

各 Chart の `templates/` には最低以下 7 種の YAML を配置する。

- `_helpers.tpl` — 共通 template 関数（`k1s0.labels`、`k1s0.selector` など）
- `deployment.yaml` — Pod 定義（spec.template.metadata に Dapr アノテーション）
- `service.yaml` — gRPC サービス公開（port 50051）
- `configmap.yaml` — 設定（OTel endpoint、log level 等）
- `serviceaccount.yaml` — ServiceAccount（最小権限）
- `networkpolicy.yaml` — Ingress / Egress 制限
- `pdb.yaml` — PodDisruptionBudget（minAvailable: 50%）

オプションで `hpa.yaml`、`servicemonitor.yaml`（Prometheus Operator）、`virtualservice.yaml`（Istio、Phase 2）を配置する。全 template は `{{- define "k1s0.fullname" . -}}` ヘルパで Pod 名を統一生成し、values の `nameOverride` で上書き可能にする。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-A-FT-\*、NFR-E-NW-\*、NFR-D-MON-\*。**上流**: DS-SW-COMP-020〜022。

### DS-IMPL-DIR-125 deployment.yaml の Dapr アノテーション

`tier1-facade/templates/deployment.yaml` は以下の Dapr アノテーションを `spec.template.metadata.annotations` に含める（facade のみ、rust は持たない）。

```yaml
annotations:
  dapr.io/enabled: "true"
  dapr.io/app-id: {{ .Values.dapr.appId }}
  dapr.io/app-port: "50051"
  dapr.io/app-protocol: "grpc"
  dapr.io/config: "k1s0-dapr-config"   # runtime/configuration/configuration.yaml
  dapr.io/log-level: "info"
  dapr.io/sidecar-cpu-request: "100m"
  dapr.io/sidecar-memory-request: "128Mi"
  dapr.io/sidecar-cpu-limit: "500m"
  dapr.io/sidecar-memory-limit: "256Mi"
  dapr.io/enable-app-health-check: "true"
```

`tier1-rust/templates/deployment.yaml` は `dapr.io/*` アノテーションを含めず、Dapr sidecar なしで稼働する（DS-SW-COMP-053 根拠）。`dapr.io/config` で参照する Configuration は `runtime/configuration/configuration.yaml` で定義する（DS-IMPL-DIR-135）。

**確定フェーズ**: Phase 1a。**対応要件**: FR-T1-\*（全 API）、NFR-B-PERF-006。**上流**: DS-SW-COMP-053、DS-SW-COMP-020。

### DS-IMPL-DIR-126 umbrella chart の dependencies

`tier1-umbrella/Chart.yaml` は以下の dependencies を宣言する。

```yaml
dependencies:
  - name: tier1-facade
    version: "0.1.0"
    repository: "file://../tier1-facade"
    condition: facade.enabled
  - name: tier1-rust
    version: "0.1.0"
    repository: "file://../tier1-rust"
    condition: rust.enabled
```

`repository: file://../tier1-facade` は同一リポジトリ内の相対パス参照（`delivery/helm/` 配下の兄弟ディレクトリ）で、`helm dependency update` で `charts/` にコピーされる。`condition` により umbrella values で `facade.enabled: false` / `rust.enabled: false` と指定して一部のみデプロイ可能にする（開発環境で Rust Pod のみ起動する等）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、DX-LD-\*。**上流**: DS-SW-COMP-053。

## delivery/kustomize/ — Kustomize overlay

### DS-IMPL-DIR-127 Kustomize と Helm の併用戦略

Helm のみでは表現しづらい環境固有の加工（特定 container への env 追加、image の pullPolicy 切替、Namespace 全体の label 付与）を Kustomize で補う。`delivery/kustomize/base/` は Helm Chart を render した出力を base とし、`delivery/kustomize/overlays/<env>/` が patch を適用する。`kustomize build --enable-helm delivery/kustomize/overlays/dev/` で Helm render + Kustomize patch を一気通貫で実行できる。

**確定フェーズ**: Phase 1b。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-OPS-ENV-\*。

### DS-IMPL-DIR-128 base/ と overlays/ の分離

`delivery/kustomize/base/kustomization.yaml` は Helm release 宣言を 1 ファイルで行う。

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
helmCharts:
  - name: tier1-umbrella
    repo: file://../../helm/tier1-umbrella
    version: 0.1.0
    releaseName: k1s0
    namespace: k1s0-tier1
    valuesFile: values.yaml
resources: []
```

各環境 overlay（`overlays/dev/` / `overlays/staging/` / `overlays/prod/`）は `kustomization.yaml` で `bases: [../../base]` を指定し、`patches/` で環境固有差分を当てる。overlay values（`values-dev.yaml`）は GitOps リポジトリに置く方針だが、開発者が手元で試すための **サンプル values** は本リポジトリの `overlays/<env>/values.yaml.sample` として git 管理する。

**確定フェーズ**: Phase 1b。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-OPS-ENV-\*。

### DS-IMPL-DIR-129 overlays/ と GitOps リポジトリの責務分離

本リポジトリの `delivery/kustomize/overlays/<env>/` は **template と patch の形だけ**を git 管理し、**実行時の変数（image tag、secret 参照、domain 名）は GitOps リポジトリに置く**。理由は以下。

- 本リポジトリは「どう組み立てるか」の設計、GitOps リポジトリは「どの環境でどう動かすか」の運用を扱う
- image tag を本リポジトリに書くと、CI が PR 毎に自動更新する機構と衝突する
- Secret 参照（OpenBao のパス）を本リポジトリに書くと、環境ごとに異なる OpenBao クラスタへの参照を単一リポジトリで管理するのが難しい

この分離はアーキテクチャ上重要で、`overlays/dev/patches/image-tag.yaml` のような動的値を含む patch は **本リポジトリでは placeholder のみ**とし、GitOps 側で最終値を注入する。

**values の Single Source of Truth（SoT）階層**: values が複数レイヤを跨ぐと「どこを書き換えれば良いか」が運用者に不明瞭になり、同じ値を複数箇所で宣言する drift が発生する。これを防ぐため、各値の種別ごとに SoT を 1 箇所に固定する。

- **静的デフォルト値**（非機密かつ環境非依存。例: Dapr sidecar のデフォルト CPU request、`dapr.io/app-port`、tracing sampling rate の baseline）: `delivery/helm/tier1-umbrella/charts/<pod>/values.yaml` を **SoT とし、overlay / GitOps では触らない**。Helm chart の変更が必ず伴う性質の値は chart 側に集約する。
- **環境別の構造差分**（dev のみ `replicas: 1`、prod のみ HPA 有効化など、**値ではなく構造**の差）: `delivery/kustomize/overlays/<env>/patches/` を SoT とし、patch 形式で宣言する。values で表現できる差は values に寄せ、patch は構造差分専用とする。
- **環境別の動的値**（image tag、ingress domain、tenant 許可リストなど、環境・時点で変わる値）: GitOps リポジトリの `values-<env>.yaml` を SoT とし、本リポジトリの Helm values には **値なしのキー定義**（`image.tag: "PLACEHOLDER"` など）のみ置く。DS-IMPL-DIR-157 の image tag placeholder ルールと整合。
- **Secret 値**: OpenBao を SoT とし、Dapr Component の `secretKeyRef` 経由で参照する（DS-IMPL-DIR-130 / DS-IMPL-DIR-158 と整合）。Helm values / Kustomize patch / GitOps values のいずれにも Secret の実値を書かない。

この階層は CI lint（`tools/check-values-sot/`、Phase 1b 後半）で機械的に強制する。lint は次の drift を検出する。(a) GitOps リポジトリの `values-<env>.yaml` に SoT が chart 側のキーが含まれている、(b) chart values に `PLACEHOLDER` 未解決のまま push されている、(c) overlay patch に動的値（環境別 URL / tag）が含まれている。

**確定フェーズ**: Phase 1b（SoT 階層の確立）、Phase 1b 後半（check-values-sot lint 追加）。**対応要件**: DX-CICD-\*、NFR-SUP-\*、NFR-C-NOP-002。**上流**: DS-OPS-ENV-\*、DS-DEVX-CICD-\*、DS-IMPL-DIR-130、DS-IMPL-DIR-157、DS-IMPL-DIR-158。

## runtime/ — Dapr 実行時設定

### DS-IMPL-DIR-130 runtime/ の配置戦略

`src/tier1/infra/runtime/` は Dapr の Component / Configuration / Subscription CRD を配置する。3 サブディレクトリ構成とする。

- `runtime/components/` — Dapr Component CRD（Building Block 別: state / pubsub / secrets / bindings / workflows）
- `runtime/configuration/` — Dapr Configuration（tracing / metric / mtls / accessControl）と Resiliency Policy
- `runtime/subscriptions/` — Declarative Subscription CRD

1 Component は 1 YAML に分離する。`runtime/` を `delivery/` から独立させたのは、Dapr Component は Pod 再デプロイと独立に Kubernetes に apply でき、更新頻度・更新主体が Pod と異なるためである。

**Secret と環境固有値の参照方式**: Dapr Component の `spec.metadata` に含まれる環境固有値は、**参照形式ごとに異なるメカニズムで注入する**。`envsubst` による `${ENV_VAR}` 展開は shell history / プロセス env / CI ログに値が露出するリスクが残るため、**本 repo では採用しない**（後述の例外のみ）。正しい参照形式を値の性質ごとに次のように固定する。

- **Secret 値**（接続パスワード / API キー / 証明書）: Dapr Component `spec.metadata[].secretKeyRef` を使い、OpenBao Secret Store Component 経由で参照する（Dapr の `secretKeyRef` 機構、DS-IMPL-DIR-133 と DS-IMPL-DIR-158 に準拠）。Kubernetes Secret に一度落とす必要がある例外ケース（一部の Operator が `secretKeyRef` ではなく Kubernetes Secret を要求する等）は External Secrets Operator（DS-IMPL-DIR-144）で OpenBao → Kubernetes Secret を同期し、Component からは `auth.secretStore: kubernetes` で Kubernetes Secret 名を参照する。**Secret 値を Helm values / Kustomize patch / 環境変数に直接書くことは禁止**（DS-IMPL-DIR-158 と整合）。
- **接続文字列 / エンドポイント URL / 非機密の metadata**（Kafka broker list / Postgres host / bucket 名など）: Helm values の `{{ .Values.xxx }}` テンプレート展開を使い、環境別の値は GitOps repo 側の `values-<env>.yaml` で上書きする。`envsubst` 相当の shell 展開は使わない。
- **tenant 別 / service 別に動的に変わる値**（`accessControl.policies` の tier2 許可リスト等）: Helm template の range ループで values から展開し、GitOps 側の `values-<env>.yaml` に集約する（DS-IMPL-DIR-135）。

**CI lint による強制**: `tools/check-dapr-components/`（Phase 1b で追加）は `runtime/components/` 配下の全 YAML を走査し、以下のパターンを検出したら PR を fail させる。(a) `spec.metadata[].value` に Secret らしき文字列（16 文字以上の高エントロピー、`password` / `token` / `key` を含む値）が直接埋め込まれている、(b) `envsubst` 用の `${ENV_VAR}` 記法が Secret 相当のキー名で使われている、(c) `secretKeyRef` を使うべき auth 設定（`auth.secretStore` が宣言された場合）でプレーン `value` が併用されている。検出パターンは `conftest` の rego で宣言し、PR の static check に組み込む。OpenBao role の存在検証（Component が参照する role が `platform/backends/security/openbao/auth-k8s.yaml` に実際に定義されているか）も同 CI で行い、Component と OpenBao 側の drift を機械的に防ぐ。

**確定フェーズ**: Phase 1b（secretKeyRef 統一とルール確立）、Phase 1b 後半（check-dapr-components lint 追加）。**対応要件**: FR-T1-STATE-\*、FR-T1-PUBSUB-\*、FR-T1-SECRETS-\*、FR-T1-BINDING-\*、NFR-E-ENC-\*、NFR-H-KEY-\*。**上流**: DS-SW-COMP-024〜029、DS-IMPL-DIR-133、DS-IMPL-DIR-144、DS-IMPL-DIR-158。

### DS-IMPL-DIR-131 components/state/ の配置

`runtime/components/state/valkey-state.yaml` は Valkey Cluster を State Store として宣言する（Phase 2）。`runtime/components/state/memory-state.yaml` は Docker ローカル開発用の in-memory バックエンドを宣言する（Phase 1a）。両者は Pod 環境変数 `DAPR_STATE_COMPONENT` で切り替え、`k1s0 dev up` 時は memory、本番は valkey を使う（DS-DEVX-LOCAL-\* と整合）。

**確定フェーズ**: Phase 1a（memory）、Phase 2（valkey）。**対応要件**: FR-T1-STATE-\*。**上流**: DS-SW-COMP-025、ADR-DATA-002。

### DS-IMPL-DIR-132 components/pubsub/ の配置

`runtime/components/pubsub/kafka-pubsub.yaml` は Strimzi Kafka を PubSub として宣言する。topic 名プレフィックス（`k1s0.`）、dead-letter topic（`k1s0.<topic>.dlq`）、retry 設定（initialInterval: 100ms、maxInterval: 30s、maxRetries: 5）を metadata で宣言する。Kafka 自体の topic 定義は `platform/backends/messaging/kafka/topics/` に配置する（DS-IMPL-DIR-146）。

**確定フェーズ**: Phase 2。**対応要件**: FR-T1-PUBSUB-\*。**上流**: DS-SW-COMP-026、ADR-MSG-001。

### DS-IMPL-DIR-133 components/secrets/ の配置

`runtime/components/secrets/openbao-secrets.yaml` は OpenBao を Secret Store として宣言する。auth 方式は `kubernetes`（k8s ServiceAccount JWT 経由で OpenBao role にログイン）を使い、各 Pod の ServiceAccount に応じた role が attach される。`platform/backends/security/openbao/auth-k8s.yaml` で Kubernetes auth method を OpenBao 側に設定する（DS-IMPL-DIR-148）。

**確定フェーズ**: Phase 1b。**対応要件**: FR-T1-SECRETS-\*、NFR-H-KEY-\*。**上流**: DS-SW-COMP-031、ADR-SEC-001。

### DS-IMPL-DIR-134 components/bindings/ の段階的配置

bindings Component は Phase 1b（HTTP binding）・Phase 1c（SMTP / S3）・Phase 2（MQTT / Dapr cron）で段階的に追加する。Phase 1a 時点では `runtime/components/bindings/` 配下は空で、`README.md` に「Phase 1b から HTTP binding を追加予定」と明記する。Binding の tenant 別許可は Policy Enforcer の `policy.rego` で制御する（[../../04_概要設計/30_共通機能方式設計/](../../04_概要設計/30_共通機能方式設計/) 参照）。

**確定フェーズ**: Phase 1b/1c/2。**対応要件**: FR-T1-BINDING-\*。**上流**: DS-SW-COMP-029。

### DS-IMPL-DIR-135 configuration/configuration.yaml の配置

`runtime/configuration/configuration.yaml` は Dapr 全体設定を宣言する。主な項目は以下。

```yaml
apiVersion: dapr.io/v1alpha1
kind: Configuration
metadata:
  name: k1s0-dapr-config
  namespace: k1s0-tier1
spec:
  tracing:
    samplingRate: "1"                 # Phase 1a は 100% sampling、Phase 2 で 10% に下げる
    otel:
      endpointAddress: "otel-collector.k1s0-monitoring.svc:4317"
      isSecure: false                 # Istio Ambient 配下なので内部は平文
      protocol: grpc
  metric:
    enabled: true
    http:
      increasedCardinality: false
  mtls:
    enabled: false                    # Istio Ambient に委譲（DS-SW-COMP-028）
  features:
    - name: PubSubPayloadEncryption
      enabled: true                   # Phase 2 で有効化
  accessControl:
    defaultAction: deny               # 明示的許可のみ通す
    trustDomain: k1s0.local
    policies: []                      # ここは Helm values で tier2 毎に生成
```

`accessControl.defaultAction: deny` は Zero Trust 方針で、各 tier2 / tier3 サービスは明示的に許可された Pod のみに Dapr 経由で到達できる（詳細は [../../04_概要設計/50_非機能方式設計/](../../04_概要設計/50_非機能方式設計/) 参照）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-D-TRACE-\*、NFR-D-MON-\*、NFR-E-NW-\*、NFR-E-AC-\*。**上流**: DS-SW-COMP-028。

### DS-IMPL-DIR-136 configuration/resiliency.yaml の配置

`runtime/configuration/resiliency.yaml` は Dapr Resiliency Policy を宣言する。retry / timeout / circuit breaker を Component（state / pubsub 等）単位で宣言し、[../../04_概要設計/40_制御方式設計/03_リトライとサーキットブレーカー方式.md](../../04_概要設計/40_制御方式設計/03_リトライとサーキットブレーカー方式.md) の方式に準拠する。Phase 1b 時点では retry 3 回 / timeout 5s / CB 閾値 50% を既定とし、Pod 別 override は Phase 1c で追加する。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-A-FT-\*、NFR-B-PERF-\*。**上流**: DS-SW-COMP-026、DS-CTRL-RETRY-\*（概要設計）。

### DS-IMPL-DIR-137 subscriptions/ の配置

`runtime/subscriptions/audit-subscription.yaml` は AUDIT Pod が `k1s0.audit.events.v1` topic を subscribe する宣言を行う。Phase 1b では audit のみ、Phase 1c で `k1s0.pii.detected.v1` topic の subscription（`pii-detected-subscription.yaml`）を追加する（PII 検出イベントの下流処理用）。subscription 定義は Pod 側のコードと一致する必要があり、CI で `tools/validate-subscriptions` を通して整合を検証する（Phase 1c）。

**確定フェーズ**: Phase 1b/1c。**対応要件**: FR-T1-AUDIT-\*、FR-T1-PII-\*。**上流**: DS-SW-COMP-054、DS-SW-COMP-070〜074。

## delivery/rollouts/ — Argo Rollouts

### DS-IMPL-DIR-138 rollouts/ の Phase 2 導入

`src/tier1/infra/delivery/rollouts/` は Argo Rollouts の Rollout CRD と AnalysisTemplate を配置する。Phase 1a 〜 1c は Deployment による全 Pod 一斉置換とし、Phase 2 から Canary 配布へ移行する。Phase 1a 時点では `rollouts/` ディレクトリに README のみ配置し、「Phase 2 で導入」と明記する。Phase 2 で導入する際は既存 Deployment を Rollout に置換する Helm template 変更と、GitOps 側での Application 更新をセットで行う（DS-IMPL-DIR-128 の base patch を書き換える）。`rollouts/` を `delivery/` 配下に置くのは、Rollout CRD は Helm Chart と連動して更新される「デプロイ成果物」であり、`runtime/`（Dapr 設定）・`governance/`（統制）・`platform/`（基盤）とは変更タイミングが異なるためである。

**確定フェーズ**: Phase 2。**対応要件**: DX-CICD-\*、NFR-A-FT-\*、NFR-A-REC-\*。**上流**: DS-DEVX-CICD-\*、DS-SW-COMP-137。

### DS-IMPL-DIR-139 AnalysisTemplate の 3 指標

`delivery/rollouts/analysis-templates/` には Canary 判定用の 3 指標を分離配置する（`error-rate.yaml` / `p99-latency.yaml` / `request-volume.yaml`）。各 template は Prometheus クエリと閾値を宣言し、Rollout manifest から `templateRefName` で参照する。閾値は Phase 2 の運用開始時に Phase 1c の実測値から 20% マージンを取って設定する。

**確定フェーズ**: Phase 2。**対応要件**: NFR-B-PERF-\*、NFR-A-FT-\*。**上流**: DS-DEVX-CICD-\*。

## governance/policies/ — Kyverno ポリシー

### DS-IMPL-DIR-140 Kyverno ClusterPolicy の 7 種

`src/tier1/infra/governance/policies/` には Kyverno ClusterPolicy を 7 種配置する。1 ファイル 1 ポリシーとし、adopt シナリオ（enforce / audit）を `spec.validationFailureAction` で明示する。

- `require-harbor-registry.yaml` — `harbor.k1s0.internal` / `ghcr.io/k1s0` 以外のレジストリ拒否（Phase 1a: audit、Phase 1b: enforce）
- `deny-latest-tag.yaml` — `:latest` / `:dev` タグ拒否（Phase 1a: enforce）
- `require-cosign-signature.yaml` — Cosign 署名検証（Phase 2: enforce）
- `require-podsecurity-restricted.yaml` — PodSecurityStandard `restricted` レベル強制（Phase 1b: enforce）
- `require-networkpolicy.yaml` — NetworkPolicy 未設定の Pod 拒否（Phase 1b: audit、Phase 1c: enforce）
- `require-resources-limits.yaml` — resources.limits 未設定の Pod 拒否（Phase 1a: audit、Phase 1b: enforce）
- `require-non-root.yaml` — `runAsNonRoot: true` 未設定の Pod 拒否（Phase 1a: enforce）

Phase ごとの audit → enforce 遷移計画は `governance/policies/README.md` に記述し、運用チームの事前検証期間を確保する。

**確定フェーズ**: Phase 1a/1b/1c/2。**対応要件**: NFR-E-AC-\*、NFR-SUP-\*、NFR-H-KEY-001。**上流**: DS-DEVX-CICD-\*、DS-NFR-SEC-\*。

### DS-IMPL-DIR-141 governance/ 配下の CODEOWNERS

`src/tier1/infra/governance/` 配下（`policies/` と `namespaces/` の両方）の変更は `@k1s0/security-team` と `@k1s0/infra-team` の両方の承認を必須とする。Policy の強化（audit → enforce、新規ポリシー追加）は全開発者への通知を伴い、影響範囲を事前アナウンスする。Policy の緩和（enforce → audit、ポリシー削除）は security team の書面承認を要する。Namespace の追加・削除も同じ権限要件とする（Namespace 変更は PodSecurityStandard やネットワーク境界に直接影響するため）。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-\*、NFR-H-\*、NFR-SUP-\*。**上流**: DS-IMPL-DIR-018。

## platform/operators/ — Operator マニフェスト

### DS-IMPL-DIR-142 operators/ の配置原則とサブグルーピング

`src/tier1/infra/platform/operators/` は Dapr / Strimzi / CloudNativePG / MinIO / OpenBao / External Secrets / Argo CD / Argo Rollouts / Kyverno / Istio の 10 Operator のインストール用マニフェストを配置する。機能グループ別に 4 サブディレクトリでサブグルーピングする。

- `operators/messaging/` — `dapr/` / `strimzi/`
- `operators/storage/` — `cnpg/` / `minio/`
- `operators/security/` — `openbao/` / `external-secrets/`
- `operators/delivery/` — `argo-cd/` / `argo-rollouts/` / `kyverno/` / `istio/`

各 Operator は専用サブディレクトリ（例: `operators/messaging/dapr/`）で、Helm values ファイル（`values.yaml`）と Helm release 宣言（`helm-release.yaml`）を対にして配置する。Operator の Helm release 自体は本リポジトリで定義し、実際の Helm install は GitOps リポジトリの Application が実行する（DS-IMPL-DIR-129 の責務分離と整合）。サブグルーピングは Phase 2 で新規基盤を追加する際の配置決定を機械的にし、CODEOWNERS の path filter（例: `platform/operators/security/` は security-team 承認追加）を書きやすくする効果がある。

**確定フェーズ**: Phase 1a（基盤 Operator）、Phase 1b（+Strimzi/CNPG/External Secrets）、Phase 1c（+MinIO）、Phase 2（+Istio/Argo Rollouts）。**対応要件**: DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-OPS-ENV-\*。

### DS-IMPL-DIR-143 Istio Ambient の Phase 2 配置

`platform/operators/delivery/istio/` は Istio Ambient Mesh を Phase 2 で導入する際の istiod / ztunnel / waypoint マニフェストを配置する。Phase 1a 時点では `istio/` ディレクトリのみ作成し、`README.md` で「Phase 2 で導入、ADR-0001 参照」と明記する。Phase 2 で mTLS と認可を Istio に委譲し、Dapr mTLS（DS-IMPL-DIR-135 の `mtls.enabled: false`）を Istio に一元化する。

**確定フェーズ**: Phase 2。**対応要件**: NFR-E-NW-\*、NFR-E-AC-\*。**上流**: ADR-0001、DS-SW-COMP-028。

### DS-IMPL-DIR-144 External Secrets の Phase 1b 導入

`platform/operators/security/external-secrets/` は External Secrets Operator を配置し、OpenBao → Kubernetes Secret への同期を実現する。Pod が直接 OpenBao を呼ぶ（Dapr Secret API 経由）のが基本だが、一部の OSS（例: CloudNativePG の superuser password）は Kubernetes Secret を要求するため、External Secrets で OpenBao → Secret へブリッジする。Phase 1b で導入し、Phase 1c で対象を拡大する。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-H-KEY-\*、NFR-E-ENC-001。**上流**: DS-OPS-ENV-\*。

## platform/backends/ — バックエンド Cluster Resource

### DS-IMPL-DIR-145 backends/ の配置原則とサブグルーピング

`src/tier1/infra/platform/backends/` はバックエンド OSS の Cluster Resource（CR）を配置する。operators/ と対応する形で 3 サブディレクトリで機能グループ別にサブグルーピングする。

- `backends/messaging/` — `kafka/`
- `backends/storage/` — `postgres/` / `valkey/` / `minio/`
- `backends/security/` — `openbao/`

各 OSS 配下に Cluster 定義・Topic / User / Database / Backup などの周辺 CR を置く。operators と backends を機能グループで対称にすることで、「messaging 領域を運用するのは `platform/operators/messaging/` と `platform/backends/messaging/` の 2 箇所」という対応が視覚的に明示される。

**CR の apply 順序 — GitOps ツール非依存の宣言**: Namespace → Operator（CRD と controller）→ Cluster（CR 本体）→ Topic/User/Database（Cluster が前提の周辺 CR）の順で段階的に反映する必要がある。Operator が CRD を登録する前に Cluster CR を apply すると `no matches for kind` で失敗するため、順序保証は必須である。ただし順序制御の**実装メカニズム**は GitOps ツール依存であり、本リポジトリはこれを抽象化する。

- **Argo CD を採用する場合**: `argocd.argoproj.io/sync-wave` annotation で wave（負の小さい値ほど先）を宣言する。本リポジトリの CR には各カテゴリに対応する wave 値を annotation で明示する（例: Namespace `-2`、Operator `-1`、Cluster `0`、周辺 CR `1`）。
- **Flux（kustomize-controller）を採用する場合**: `Kustomization` CR の `spec.dependsOn` で依存元の Kustomization を宣言する。本リポジトリでは機能グループ（messaging / storage / security）単位に `flux/kustomizations/` ディレクトリを別立てし、`dependsOn` を宣言する（Phase 1b 時点では Argo CD 採用を前提とするが、ツール切替時の作業量を小さく保つ設計）。
- **CR 本体に含めるメタデータ**: GitOps ツールに固有な `sync-wave` annotation は CR 本体には埋め込まず、**overlay 層で Kustomize patch により注入する**。CR 本体は「Cluster をどう宣言するか」のみを表現し、「どの順序で apply するか」は GitOps レイヤの責務とする。これにより CR 本体の YAML は GitOps ツール中立に保たれ、ツール切替時の影響を overlay に封じ込められる。

この分離は DS-IMPL-DIR-129 の values SoT 階層と同じ発想で、「どう組み立てるか（本リポジトリ）」と「どう apply するか（GitOps リポジトリ）」の責務を明確に分ける。Phase 1b 時点の採用ツールは ADR-DELIVERY-001（Argo CD 採用）で確定しているが、本設計上は差替え可能な前提で書く。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-A-FT-\*、NFR-A-REC-\*、NFR-C-NOP-002、NFR-SUP-\*。**上流**: DS-OPS-ENV-\*、DS-SW-COMP-054、ADR-DELIVERY-001、DS-IMPL-DIR-129。

### DS-IMPL-DIR-146 Kafka の topic / user 配置

`platform/backends/messaging/kafka/topics/` は Strimzi KafkaTopic CR を配置する。上流 [04_概要設計/10_システム方式設計/04_データベース方式設計.md](../../04_概要設計/10_システム方式設計/04_データベース方式設計.md) および [ADR-DATA-002](../../02_構想設計/adr/ADR-DATA-002-strimzi-kafka.md) で Kafka は **Phase 2 で導入**と確定しており、Phase 1a/1b 期間中は `platform/backends/messaging/kafka/` ディレクトリは README のみを置き CR 本体は未配置とする（物理的にスケルトンだけ存在させる目的は、Phase 2 移行時の PR が「ファイル追加のみで `kubectl apply -k` を変えない」で済むようにするため）。Phase 2 で配置する topic は以下。

- `audit-events.yaml` — topic `k1s0.audit.events.v1`（partition 10、replication 3、retention 90d）
- `pii-detected.yaml` — topic `k1s0.pii.detected.v1`（partition 5、replication 3、retention 7d）

将来の topic 追加は同ディレクトリに追加する。`platform/backends/messaging/kafka/users/tier1-users.yaml` は KafkaUser CR で、AUDIT Pod の Consumer ユーザ・PII Pod の Producer ユーザを分離宣言する。認証は mTLS（Strimzi 標準）、認可は topic 別に ACL を付与する。Phase 1b までの AUDIT / PII Pod は Kafka 非依存の一時経路（stdout → Loki 経由の audit 取込、PII 検出は同期 Pod 内処理）で稼働させ、Phase 2 で Kafka 経路に切替える。切替は ADR を要する。

**確定フェーズ**: Phase 2（Kafka 本体導入に合わせて）、Phase 1a（README スケルトンのみ）。**対応要件**: FR-T1-AUDIT-\*、FR-T1-PII-\*、NFR-E-AC-\*。**上流**: DS-SW-COMP-054、DS-SW-COMP-070〜074、ADR-DATA-002、DS-SYS-DB-005。

### DS-IMPL-DIR-147 PostgreSQL の Cluster CR と Backup

`platform/backends/storage/postgres/audit-cluster.yaml` は CNPG の Cluster CR で、3 node + 1 synchronous replica + PITR 対応を宣言する。`platform/backends/storage/postgres/scheduled-backups/audit-backup.yaml` は ScheduledBackup CR で、日次 full backup を MinIO に保存する設定を宣言する。Backup の保持期間は 90 日（audit データと同期）。Postgres の認証情報は External Secrets で OpenBao から同期する。

**確定フェーズ**: Phase 1b。**対応要件**: FR-T1-AUDIT-\*、NFR-A-REC-\*、NFR-H-INT-\*。**上流**: DS-SW-COMP-054〜056。

### DS-IMPL-DIR-148 OpenBao の Cluster CR と auth

`platform/backends/security/openbao/openbao-cluster.yaml` は OpenBao の 3 node cluster CR で、Raft Integrated Storage を使う。`platform/backends/security/openbao/auth-k8s.yaml` は Kubernetes Auth method 有効化と、Pod 別 role 定義を含む。role は tenant × Pod（例: `tenant-a-state-role`）単位で細分化し、最小権限原則を徹底する。OpenBao の unseal 手順は Phase 1c で自動化（Kubernetes Secret + awskms 準拠機構）を検討する。

**確定フェーズ**: Phase 1b（基本）、Phase 1c（unseal 自動化）。**対応要件**: NFR-H-KEY-\*、NFR-E-ENC-001。**上流**: DS-SW-COMP-031、ADR-SEC-001。

### DS-IMPL-DIR-149 Valkey Cluster CR

`platform/backends/storage/valkey/valkey-cluster.yaml` は Valkey Cluster（3 master + 3 replica の 6 node）の CR を配置する。上流 [04_概要設計/10_システム方式設計/04_データベース方式設計.md](../../04_概要設計/10_システム方式設計/04_データベース方式設計.md) で Valkey も Kafka と同じく **Phase 2 で導入**と確定しており、Phase 1a/1b 期間中の STATE Pod は Dapr `state.in-memory` で稼働する（[DS-IMPL-DIR-131](#ds-impl-dir-131) の Component が使い分ける）。`platform/backends/storage/valkey/` ディレクトリは Phase 1a から README のみを置き、CR 本体は Phase 2 で配置する。Valkey の Kubernetes Operator は Phase 2 時点でも stable 運用が不十分と見込まれるため、Helm chart ベースのインストール（`platform/operators/storage/valkey/` 配下）を採用する。`values.yaml` で cluster 構成を宣言し、本 CR はその Helm release 宣言の形を取る。Valkey のデータ永続化は `emptyDir`（in-memory 前提、データロスト可）ではなく PersistentVolume を使い、Phase 2 の導入時点で AOF / RDB 両方式のバックアップを有効化する。

**確定フェーズ**: Phase 2（Valkey 本体導入に合わせて）、Phase 1a（README スケルトンのみ）。**対応要件**: FR-T1-STATE-\*、NFR-A-REC-\*。**上流**: DS-SW-COMP-025、ADR-DATA-004、DS-SYS-DB-004。

### DS-IMPL-DIR-150 MinIO Tenant CR

`platform/backends/storage/minio/minio-tenant.yaml` は MinIO Operator の Tenant CR で、Object Lock 対応の bucket を 3 種類宣言する（audit-archive / kafka-backup / postgres-backup）。Object Lock の retention は audit 10 年（法定最長）、他は 90 日。MinIO access/secret key は External Secrets で OpenBao から同期する。

**確定フェーズ**: Phase 1c。**対応要件**: FR-T1-AUDIT-\*、NFR-H-INT-\*、NFR-A-REC-\*。**上流**: DS-SW-COMP-054（MinIO archiver）。

## governance/namespaces/ — Namespace

### DS-IMPL-DIR-151 Namespace 分離戦略

`src/tier1/infra/governance/namespaces/` は以下 6 Namespace を配置する。

- `k1s0-tier1` — tier1 の 6 Pod（facade 3 + rust 3）
- `k1s0-dapr` — Dapr Operator と control plane
- `k1s0-kafka` — Strimzi Operator と Kafka Cluster
- `k1s0-postgres` — CNPG Operator と PG Cluster
- `k1s0-openbao` — OpenBao Cluster
- `k1s0-monitoring` — Prometheus / Grafana / Loki / Tempo / OTel Collector

各 Namespace には PodSecurityStandard の `restricted` レベルを label で付与（`pod-security.kubernetes.io/enforce: restricted`）し、最小権限 Pod のみを許可する。Namespace 間通信は NetworkPolicy で明示許可のみ通す。Namespace を `governance/` 配下に置くのは、Namespace 自体が統制境界（PodSecurityStandard・NetworkPolicy の enforcement 単位）であるためで、`platform/` の Operator・Backend とは責務レイヤが異なるためである。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-NW-\*、NFR-E-AC-\*。**上流**: DS-OPS-ENV-\*、DS-NFR-SEC-\*。

### DS-IMPL-DIR-152 Namespace YAML の label / annotation

各 Namespace YAML は以下の label / annotation を必須とする。

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: k1s0-tier1
  labels:
    app.kubernetes.io/part-of: k1s0
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
    istio.io/dataplane-mode: ambient      # Phase 2 の Istio Ambient
  annotations:
    k1s0.internal/owner: "tier1-architects"
    k1s0.internal/cost-center: "tier1-platform"
```

`istio.io/dataplane-mode: ambient` は Phase 2 で Istio Ambient に組み込む際に効く label。`cost-center` annotation は Phase 2 の cost metering（DS-DEVX-BIZ-\*）で使う。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-E-AC-\*、BC-COST-\*（Phase 2）。**上流**: DS-OPS-ENV-\*。

## CI と連携

### DS-IMPL-DIR-153 helm lint の CI 組込

PR パイプライン（`.github/workflows/pr-infra.yml`、[07_開発ツールとCICD配置.md](07_開発ツールとCICD配置.md) で定義）で以下を実行する。path filter は `src/tier1/infra/delivery/helm/**` を対象とする。

1. `helm lint src/tier1/infra/delivery/helm/tier1-facade`
2. `helm lint src/tier1/infra/delivery/helm/tier1-rust`
3. `helm lint src/tier1/infra/delivery/helm/tier1-umbrella`
4. `helm template ... | kubeconform -strict -summary` — Kubernetes スキーマ検証
5. `helm template ... --values test/values-dev.yaml` — values.schema.json 違反検出

`test/values-dev.yaml` は `delivery/helm/<chart>/tests/` 配下に配置し、開発環境相当の values サンプルを提供する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-DEVX-CICD-\*。

### DS-IMPL-DIR-154 conftest によるポリシー CI

Kyverno ClusterPolicy は cluster apply で効くが、PR 段階で違反を検出するため `conftest` で同等の検証を CI で実行する。`src/tier1/infra/governance/policies/` 配下を rego 形式に変換（Phase 1b で `tools/kyverno-to-rego/` を追加）し、`helm template` の出力に対して `conftest test` を実行する。これにより「Helm チャートの変更が Kyverno で拒否される」を PR 段階で発見できる。

**確定フェーズ**: Phase 1b。**対応要件**: DX-CICD-\*、NFR-E-AC-\*。**上流**: DS-DEVX-CICD-\*。

### DS-IMPL-DIR-155 infra 変更の Canary 検証

infra 変更（Helm chart / Kustomize overlay / Dapr Component / Kyverno ポリシー）の本番適用は、dev → staging → prod の段階 deployment を必須とする。各段階で GitOps リポジトリの Application sync が成功 + smoke test pass を確認してから次段階に進む。Phase 2 で Argo Rollouts が導入されれば infra 側も Canary 可能だが、Phase 1c 時点では Application 粒度の段階 sync で代替する。段階 sync の対象は 4 カテゴリ別に分け、特に `governance/policies/` の強化（audit → enforce 遷移）は独立した PR で分離し、Pod の変更と同時 apply しない。

**確定フェーズ**: Phase 1b。**対応要件**: DX-CICD-\*、NFR-A-REC-\*。**上流**: DS-OPS-ENV-\*。

## その他の運用規約

### DS-IMPL-DIR-156 infra/ 配下の CODEOWNERS

`src/tier1/infra/` 配下の変更は `@k1s0/infra-team` が責任を持つが、カテゴリ別に追加承認者を規定する。

- `delivery/helm/tier1-facade/` / `delivery/helm/tier1-rust/` の `templates/deployment.yaml` など tier1 Pod の挙動に直接影響する箇所: `@k1s0/tier1-architects` も承認必須
- `runtime/`: `@k1s0/tier1-architects`（Dapr Component / Configuration は Pod の挙動を直接左右するため）
- `governance/`: `@k1s0/security-team` + `@k1s0/infra-team` の両承認（DS-IMPL-DIR-141）
- `platform/operators/security/` / `platform/backends/security/`: `@k1s0/security-team` も承認必須
- `platform/operators/` / `platform/backends/` その他: `@k1s0/infra-team` のみ

カテゴリ分離により CODEOWNERS の path filter が単純化され、承認権限の責務が明瞭になる。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-C-NOP-001、DX-CICD-\*。**上流**: DS-IMPL-DIR-018。

### DS-IMPL-DIR-157 image tag の placeholder ルール

Helm values や Kustomize overlay で image tag を指定する場合、本リポジトリでは以下 3 種類の placeholder のみを許可する。

- `latest-dev` — dev 環境用の moving tag（Argo CD が自動更新）
- `{{ .Values.image.tag }}` — Helm template
- `<GITOPS_PLACEHOLDER>` — GitOps リポジトリで置換するマーカー

具体的な semver + git hash（`v0.1.0-abc123`）を本リポジトリに commit することは禁止する（GitOps の image tag 自動更新と競合するため）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-SUP-\*。**上流**: DS-IMPL-DIR-129。

### DS-IMPL-DIR-158 Secret の placeholder ルール

Helm values や Kustomize overlay で Secret 値を直接記述することは禁止する。必ず以下のいずれかの参照形式を使う。

- `valueFrom.secretKeyRef` — Kubernetes Secret 参照（Deployment / Pod の env 注入）
- External Secrets の `secretStoreRef` — OpenBao 参照（Kubernetes Secret を OpenBao から同期生成）
- `envFrom.secretRef` — Secret 全体の env 注入
- Dapr Component の `spec.metadata[].secretKeyRef` — Component metadata 内の Secret 参照（DS-IMPL-DIR-130 と整合）

「開発用の仮パスワード」を values に書くことも禁止する（commit 履歴に残り、後で本番に漏れるリスク）。dev 環境用の仮 Secret は GitOps リポジトリの dev overlay で External Secrets 経由で OpenBao Dev Mode から取得する。**`envsubst` による `${ENV_VAR}` 展開で Secret 値を注入するパターンも禁止**する（shell history・CI ログ・プロセス env への露出リスク）。

**CI による機械的 enforcement**: Phase 1b で `tools/check-secret-hygiene/`（または既存 `tools/check-deps` に機能追加）を導入し、以下のパターンを PR で検出したら自動 fail させる。(1) `values.yaml` / `overlays/*/patches/*.yaml` / `runtime/components/*.yaml` / `delivery/helm/*/templates/*.yaml` 配下に 16 文字以上の高エントロピー文字列が直接値として現れる、(2) `password` / `token` / `apikey` / `secret` / `credential` をキー名に含む field に `valueFrom` / `secretKeyRef` / `secretStoreRef` 以外の参照形式が使われる、(3) `env:` 配列内の `name` が機密キー名（上記パターン）で `valueFrom` 以外を使う。検出ルールは `conftest` の rego で宣言し、Kyverno ポリシー（`governance/policies/`）とは別レイヤで PR 段階の防御とする（cluster 側の Kyverno は apply 時、CI 側の conftest は PR 時の二重防御）。

**参照チェーンの両方向 lint**: OpenBao が提供する role / secret path と、Component / Deployment 側の参照は必ず対応が取れていなければならない。`tools/check-secret-hygiene/` は Phase 1b 後半で**双方向検証**を追加する。(a) Component の `secretKeyRef.key` が OpenBao `auth-k8s.yaml` の role に対応する secret path に実在すること、(b) OpenBao 側の role が実際に Pod の ServiceAccount から参照されるよう Kubernetes auth binding が設定されていること。drift があれば PR を fail させ、開発者に片側の追加漏れを通知する。

**確定フェーズ**: Phase 1a（ルール）、Phase 1b（CI lint 導入）、Phase 1b 後半（双方向検証）。**対応要件**: NFR-E-ENC-\*、NFR-H-KEY-\*、NFR-SUP-\*。**上流**: DS-SW-COMP-031、DS-IMPL-DIR-129、DS-IMPL-DIR-130、DS-IMPL-DIR-148。

### DS-IMPL-DIR-159 resource requests / limits の必須化

全 Deployment / StatefulSet は `resources.requests` と `resources.limits` を必須とし、Kyverno `require-resources-limits.yaml` で enforce する。requests / limits の値は Pod 別に決定し、Phase 1a 時点の Rust bin Pod は `cpu: 200m/1000m, memory: 256Mi/1Gi`、Go facade Pod は `cpu: 100m/500m, memory: 128Mi/512Mi`、Dapr sidecar は values で `100m/500m, 128Mi/256Mi` を既定とする。実測後に Phase 1c で再チューニングする。

**確定フェーズ**: Phase 1a。**対応要件**: NFR-B-PERF-\*、NFR-F-ENV-\*。**上流**: DS-SW-COMP-052（tokio runtime）。

### DS-IMPL-DIR-160 infra 変更の ADR 起票条件

以下の変更は ADR 起票を要する。

1. Helm Chart の分割・統合（例: `tier1-facade` を Pod 別 Chart に分割）
2. Kustomize overlay 戦略の変更（例: base 共通化の境界変更）
3. 4 カテゴリ（`delivery/` / `runtime/` / `governance/` / `platform/`）の境界変更、または新カテゴリ追加
4. 新規 Operator の追加（DS-IMPL-DIR-142 の 10 Operator 以外）
5. Kyverno ポリシーの新規追加（DS-IMPL-DIR-140 の 7 種以外）
6. Namespace の新規追加（DS-IMPL-DIR-151 の 6 Namespace 以外）
7. バックエンド OSS の追加（DS-IMPL-DIR-145 の 5 OSS 以外）
8. `platform/operators/` または `platform/backends/` のサブグルーピング（messaging / storage / security / delivery）変更

軽微な変更（values.yaml の数値調整、template の label 追加）は ADR 不要だが、CODEOWNERS レビューは必須。

**確定フェーズ**: Phase 0（ルール）、各 Phase（適用）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*。**上流**: DS-SW-COMP-138、DS-IMPL-DIR-018。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 配置カテゴリ | 確定フェーズ |
|---|---|---|---|
| DS-IMPL-DIR-121 | Helm Chart 3 枚体制 | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-122 | Chart.yaml の必須フィールド | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-123 | values.schema.json の必須化 | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-124 | templates/ の必須 YAML 7 種 | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-125 | deployment.yaml の Dapr アノテーション | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-126 | umbrella chart の dependencies | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-127 | Kustomize と Helm の併用戦略 | delivery/kustomize/ | Phase 1b |
| DS-IMPL-DIR-128 | base/ と overlays/ の分離 | delivery/kustomize/ | Phase 1b |
| DS-IMPL-DIR-129 | overlays/ と GitOps リポジトリの責務分離 | delivery/kustomize/ | Phase 1b |
| DS-IMPL-DIR-130 | runtime/ の配置戦略（3 サブディレクトリ） | runtime/ | Phase 1b |
| DS-IMPL-DIR-131 | components/state/ の配置 | runtime/components/ | Phase 1a/2 |
| DS-IMPL-DIR-132 | components/pubsub/ の配置 | runtime/components/ | Phase 2 |
| DS-IMPL-DIR-133 | components/secrets/ の配置 | runtime/components/ | Phase 1b |
| DS-IMPL-DIR-134 | components/bindings/ の段階的配置 | runtime/components/ | Phase 1b/1c/2 |
| DS-IMPL-DIR-135 | configuration/configuration.yaml | runtime/configuration/ | Phase 1b |
| DS-IMPL-DIR-136 | configuration/resiliency.yaml | runtime/configuration/ | Phase 1b |
| DS-IMPL-DIR-137 | subscriptions/ の配置 | runtime/subscriptions/ | Phase 1b/1c |
| DS-IMPL-DIR-138 | rollouts/ の Phase 2 導入 | delivery/rollouts/ | Phase 2 |
| DS-IMPL-DIR-139 | AnalysisTemplate の 3 指標 | delivery/rollouts/ | Phase 2 |
| DS-IMPL-DIR-140 | Kyverno ClusterPolicy の 7 種 | governance/policies/ | Phase 1a〜2 |
| DS-IMPL-DIR-141 | governance/ 配下の CODEOWNERS | governance/ | Phase 1a |
| DS-IMPL-DIR-142 | operators/ の配置原則（4 サブグループ） | platform/operators/ | Phase 1a〜2 |
| DS-IMPL-DIR-143 | Istio Ambient の Phase 2 配置 | platform/operators/delivery/ | Phase 2 |
| DS-IMPL-DIR-144 | External Secrets の Phase 1b 導入 | platform/operators/security/ | Phase 1b |
| DS-IMPL-DIR-145 | backends/ の配置原則（3 サブグループ） | platform/backends/ | Phase 1b |
| DS-IMPL-DIR-146 | Kafka の topic / user 配置 | platform/backends/messaging/ | Phase 2（スケルトン: Phase 1a） |
| DS-IMPL-DIR-147 | PostgreSQL の Cluster CR と Backup | platform/backends/storage/ | Phase 1b |
| DS-IMPL-DIR-148 | OpenBao の Cluster CR と auth | platform/backends/security/ | Phase 1b/1c |
| DS-IMPL-DIR-149 | Valkey Cluster CR | platform/backends/storage/ | Phase 2（スケルトン: Phase 1a） |
| DS-IMPL-DIR-150 | MinIO Tenant CR | platform/backends/storage/ | Phase 1c |
| DS-IMPL-DIR-151 | Namespace 分離戦略 | governance/namespaces/ | Phase 1a |
| DS-IMPL-DIR-152 | Namespace YAML の label / annotation | governance/namespaces/ | Phase 1a |
| DS-IMPL-DIR-153 | helm lint の CI 組込 | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-154 | conftest によるポリシー CI | governance/policies/ | Phase 1b |
| DS-IMPL-DIR-155 | infra 変更の Canary 検証 | 4 カテゴリ横断 | Phase 1b |
| DS-IMPL-DIR-156 | infra/ 配下の CODEOWNERS（カテゴリ別） | 4 カテゴリ横断 | Phase 1a |
| DS-IMPL-DIR-157 | image tag の placeholder ルール | delivery/ | Phase 1a |
| DS-IMPL-DIR-158 | Secret の placeholder ルール | delivery/, runtime/ | Phase 1a |
| DS-IMPL-DIR-159 | resource requests / limits の必須化 | delivery/helm/ | Phase 1a |
| DS-IMPL-DIR-160 | infra 変更の ADR 起票条件（カテゴリ境界含む） | 4 カテゴリ横断 | Phase 0 |

### 対応要件一覧

- NFR-A-FT-\*、NFR-A-REC-\*、NFR-B-PERF-\*、NFR-C-NOP-001、NFR-C-NOP-002、NFR-D-MON-\*、NFR-D-TRACE-\*、NFR-E-AC-\*、NFR-E-ENC-\*、NFR-E-NW-\*、NFR-F-ENV-\*、NFR-G-PROT-\*、NFR-H-INT-\*、NFR-H-KEY-\*、NFR-SUP-\*
- FR-T1-\*（全 API）
- DX-CICD-\*、BC-COST-\*（Phase 2）

### 上流設計 ID

DS-SW-COMP-020〜022（ファサード 5 モジュール）、DS-SW-COMP-024〜031（Dapr BB 別内部モジュール）、DS-SW-COMP-053（Dapr sidecar 不要）、DS-SW-COMP-054（Audit の 5 モジュール）、DS-SW-COMP-070〜074（PII）、DS-SW-COMP-120（tier1 トップレベル）、DS-SW-COMP-137（コンテナレジストリ）、DS-SW-COMP-138（変更手続）、DS-OPS-ENV-\*（環境構成管理）、DS-DEVX-CICD-\*（CI/CD）、DS-NFR-SEC-\*（セキュリティ）、ADR-0001（Istio Ambient）、ADR-DATA-002（Valkey）、ADR-MSG-001（Strimzi Kafka）、ADR-SEC-001（OpenBao）。DS-IMPL-DIR-018（CODEOWNERS）と双方向トレースする。
