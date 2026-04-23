# 02. ArgoCD ApplicationSet 配置

本ファイルは `deploy/apps/` 配下の ArgoCD Application / ApplicationSet 配置を確定する。App-of-Apps パターンで全配信を 1 ルート Application から展開する。

![ArgoCD ApplicationSet 展開](img/ArgoCD_ApplicationSet展開.svg)

## App-of-Apps パターンの採用理由

ArgoCD では 100 個超の Application を運用することが珍しくない。個別管理は煩雑になるため、以下の階層を採用する。

1. **ルート Application**（`apps/app-of-apps.yaml`）: 1 個、ArgoCD 初期化時に手動で apply
2. **ApplicationSet**（`apps/application-sets/*.yaml`）: カテゴリ毎（infra / tier1 / tier2 / tier3 / ops）、ルートから展開
3. **個別 Application**: ApplicationSet が生成する（git-generator / list-generator で）

この階層により、新 tier1 サービス追加時も ApplicationSet が自動検知する。

## レイアウト詳細

```
deploy/apps/
├── README.md
├── app-of-apps.yaml            # 最上位 Application
├── application-sets/
│   ├── infra.yaml              # infra/environments/<env>/ 全体
│   ├── tier1.yaml              # src/tier1/go + src/tier1/rust の 6 Pod
│   ├── tier2.yaml              # src/tier2/dotnet/services/ + src/tier2/go/services/
│   ├── tier3.yaml              # src/tier3/web/apps/ + src/tier3/bff + native は image 配布外
│   └── ops.yaml                # ops/chaos/ 等の Runbook 起動定義
└── projects/
    ├── k1s0-platform.yaml      # AppProject 定義
    └── rbac.yaml               # SSO 連携 RBAC
```

## app-of-apps.yaml

ArgoCD が最初に Sync するルート。子 ApplicationSet 群を inspect する。

```yaml
# apps/app-of-apps.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: app-of-apps
  namespace: argocd
spec:
  project: k1s0-platform
  source:
    repoURL: https://github.com/k1s0/k1s0.git
    targetRevision: HEAD
    path: deploy/apps/application-sets
    directory:
      recurse: true
  destination:
    server: https://kubernetes.default.svc
    namespace: argocd
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

## ApplicationSet の generator 戦略

### infra.yaml（list-generator）

環境を固定（dev / staging / prod）するため list-generator を使う。

```yaml
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: infra
  namespace: argocd
spec:
  generators:
    - list:
        elements:
          - env: dev
            cluster: https://dev.k1s0.internal
          - env: staging
            cluster: https://staging.k1s0.internal
          - env: prod
            cluster: https://prod.k1s0.internal
  template:
    metadata:
      name: infra-{{env}}
    spec:
      project: k1s0-platform
      source:
        repoURL: https://github.com/k1s0/k1s0.git
        targetRevision: HEAD
        path: infra/environments/{{env}}
      destination:
        server: "{{cluster}}"
        namespace: ""
```

### tier1.yaml / tier2.yaml / tier3.yaml（git-generator）

新サービス追加を自動検知するため git-generator を使う。

```yaml
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: tier2
  namespace: argocd
spec:
  generators:
    - matrix:
        generators:
          - git:
              repoURL: https://github.com/k1s0/k1s0.git
              revision: HEAD
              directories:
                - path: deploy/charts/tier2-*/
          - list:
              elements:
                - env: dev
                - env: staging
                - env: prod
  template:
    metadata:
      name: "{{path.basename}}-{{env}}"
    spec:
      project: k1s0-platform
      source:
        repoURL: https://github.com/k1s0/k1s0.git
        targetRevision: HEAD
        path: "{{path}}"
        helm:
          valueFiles:
            - values.yaml
            - ../overlays/{{env}}/values.yaml
```

matrix generator で「新サービス追加」「環境展開」の 2 次元を自動生成する。

## AppProject の RBAC

`projects/k1s0-platform.yaml` で operation の権限境界を規定する。

```yaml
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: k1s0-platform
  namespace: argocd
spec:
  description: k1s0 全体プロジェクト
  sourceRepos:
    - https://github.com/k1s0/k1s0.git
  destinations:
    - namespace: '*'
      server: https://dev.k1s0.internal
    - namespace: '*'
      server: https://staging.k1s0.internal
    - namespace: '*'
      server: https://prod.k1s0.internal
  clusterResourceWhitelist:
    - group: '*'
      kind: '*'
  roles:
    - name: admin
      policies:
        - p, proj:k1s0-platform:admin, applications, *, k1s0-platform/*, allow
      groups:
        - k1s0:sre-ops
    - name: developer
      policies:
        - p, proj:k1s0-platform:developer, applications, get, k1s0-platform/*, allow
        - p, proj:k1s0-platform:developer, applications, sync, k1s0-platform/tier2-*-dev, allow
      groups:
        - k1s0:tier2-dev
        - k1s0:tier3-web
```

SRE は全環境 admin、開発者は dev 環境の担当サービスのみ Sync 可能。

## Sync Wave

ApplicationSet で `annotations: argocd.argoproj.io/sync-wave` を設定、依存順に展開:

- Wave -10: infra/k8s/bootstrap
- Wave -5: infra/mesh / infra/dapr / infra/security / infra/observability
- Wave 0: infra/data
- Wave 10: tier1
- Wave 20: tier2
- Wave 30: tier3

起動順序を明示的に制御する。

## 対応 IMP-DIR ID

- IMP-DIR-OPS-092（ArgoCD ApplicationSet 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-CICD-001（GitOps）
- ADR-CICD-002（ArgoCD）
- DX-CICD-\* / NFR-C-NOP-\*
