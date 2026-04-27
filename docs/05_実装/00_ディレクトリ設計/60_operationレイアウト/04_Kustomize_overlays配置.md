# 04. Kustomize overlays 配置

本ファイルは `deploy/kustomize/` 配下の base + overlays 構成を確定する。Helm chart で吸収しきれない環境差分やサービス固有の patch を Kustomize で表現する。

## Helm と Kustomize の使い分け

`deploy/charts/` の Helm chart と `deploy/kustomize/` の Kustomize overlay は共存する。役割分担:

- **Helm**: テンプレート言語でパラメタ化された generic 定義。tier1-facade のような「同型サービスが 10 個ある」場合に威力
- **Kustomize**: strategic merge / JSON patch による差分適用。環境依存の上書き、1 回性の patch、Helm でも表現しにくい複雑な変更

ArgoCD は Helm と Kustomize を同時サポート。ApplicationSet で `helm:` + `kustomize:` を併用可能。

## レイアウト

```text
deploy/kustomize/
├── README.md
├── base/                       # Helm で吸収しきれない共通定義
│   ├── tier1/
│   │   ├── kustomization.yaml
│   │   ├── configmap-common.yaml    # 全 tier1 共通 config
│   │   └── rbac.yaml                 # ServiceAccount / Role
│   ├── tier2/
│   │   ├── kustomization.yaml
│   │   └── configmap-common.yaml
│   ├── tier3/
│   │   └── ...
│   └── shared/
│       ├── kustomization.yaml
│       └── network-policies.yaml
└── overlays/
    ├── dev/
    │   ├── kustomization.yaml
    │   ├── image-transformers/
    │   │   └── images.yaml           # image repo を harbor-dev に置換
    │   ├── patches/
    │   │   ├── reduce-replicas.yaml  # replica 3 → 1
    │   │   └── debug-logging.yaml    # log level DEBUG
    │   └── resources/
    │       └── ...                    # dev 専用追加 resource
    ├── staging/
    │   ├── kustomization.yaml
    │   ├── patches/
    │   └── resources/
    └── prod/
        ├── kustomization.yaml
        ├── patches/
        └── resources/
```

## base/ の位置付け

`base/` は「全環境共通で、Helm chart の外に置く YAML」だけを集める。例: tier1 全体で共通の ConfigMap、ServiceAccount、RBAC 定義。

Helm chart が管理する Deployment / Service は base/ には含めない。

## overlays/ の構造

### image-transformers

環境ごとに image レジストリを差し替える。

```yaml
# overlays/dev/image-transformers/images.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

images:
  - name: harbor.k1s0.internal/tier1/facade
    newName: harbor-dev.k1s0.internal/tier1/facade
    newTag: dev-latest
  - name: harbor.k1s0.internal/tier2/payroll
    newName: harbor-dev.k1s0.internal/tier2/payroll
    newTag: dev-latest
```

### patches

strategic merge で base の一部を書換える。

```yaml
# overlays/dev/patches/reduce-replicas.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tier1-facade-service-invoke
spec:
  replicas: 1
  template:
    spec:
      containers:
        - name: tier1-facade
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
```

### resources

環境専用リソース（例: dev だけに Port-Forward 用 NodePort Service）。

## kustomization.yaml の例

```yaml
# overlays/prod/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: k1s0-tier1

resources:
  - ../../base/tier1
  - ../../base/shared
  - resources/pdb-strict.yaml

patches:
  - path: patches/resource-prod.yaml
    target:
      kind: Deployment
      labelSelector: app.kubernetes.io/part-of=k1s0-tier1

commonLabels:
  environment: prod

configMapGenerator:
  - name: tier1-common
    behavior: merge
    literals:
      - LOG_LEVEL=INFO
      - ENVIRONMENT=prod
```

## Helm + Kustomize のハイブリッド運用

ArgoCD Application の例:

```yaml
# deploy/apps/application-sets/tier1.yaml (抜粋)
spec:
  source:
    repoURL: https://github.com/k1s0/k1s0.git
    targetRevision: HEAD
    path: deploy/charts/tier1-facade
    helm:
      valueFiles:
        - values.yaml
    kustomize:
      namePrefix: ""
      images:
        - name: harbor.k1s0.internal/tier1/facade
          newTag: "{{image-tag}}"
```

この場合、ArgoCD は「helm template → kustomize build」の 2 段階で YAML を生成。

> 注: `helm` と `kustomize` を同一 `source` 内で併用する設定は ArgoCD 2.4+ で `kustomize.buildOptions` に `--enable-helm` を指定することで有効化される（それ以前のバージョンでは `helm` と `kustomize` は相互排他）。k1s0 は ArgoCD 2.11+ を前提とし、`argocd-cm` ConfigMap で `kustomize.buildOptions: --enable-helm` を設定する。hybrid 運用は debug が困難になる（どの層で値が上書きされたかが追跡しにくい）ため、まずは Helm values で完結するケースを優先し、kustomize layer は image 差し替えと environment-specific patch に限定する。hybrid を避けたい場合は、Helm chart 側に `image.tag` を values として露出し、`values-<env>.yaml` で完結させる。

## 対応 IMP-DIR ID

- IMP-DIR-OPS-094（Kustomize overlays 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-CICD-005（Kustomize 採用）
- DX-CICD-\*
