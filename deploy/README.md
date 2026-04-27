# deploy — k1s0 GitOps 配信定義

本ディレクトリは Argo CD / Argo Rollouts / Helm chart / Kustomize / OpenTofu の配信ソースを統合する。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/01_deploy配置_GitOps.md`](../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/01_deploy配置_GitOps.md)。

## 配置構造

```text
deploy/
├── README.md                           # 本ファイル
├── apps/                               # Argo CD Application / ApplicationSet
│   ├── app-of-apps.yaml                # ルート Application（手動 apply の唯一）
│   ├── application-sets/               # カテゴリ別 ApplicationSet
│   │   ├── infra.yaml                  # infra/environments/<env>/ 配備（Wave -10）
│   │   ├── ops.yaml                    # ops（Backstage / chaos / runbook）配備（Wave 40〜45）
│   │   ├── tier1-facade.yaml           # tier1 Go 3 Pod
│   │   ├── tier1-rust-service.yaml     # tier1 Rust 3 Pod
│   │   ├── tier2-dotnet-service.yaml   # tier2 .NET ドメインサービス
│   │   ├── tier2-go-service.yaml       # tier2 Go ドメインサービス
│   │   ├── tier3-bff.yaml              # tier3 BFF（portal-bff / admin-bff）
│   │   └── tier3-web-app.yaml          # tier3 Web SPA（portal / admin / docs-site）
│   └── projects/                       # AppProject（RBAC 境界）
│       ├── k1s0-platform.yaml          # umbrella（infra / ops / GitOps ルート）
│       ├── k1s0-tier1.yaml             # tier1 専用境界
│       ├── k1s0-tier2.yaml             # tier2 専用境界
│       ├── k1s0-tier3.yaml             # tier3 専用境界
│       └── rbac.yaml                   # Argo CD グローバル RBAC（OIDC 連携）
├── charts/                             # Helm chart 6 種
│   ├── tier1-facade/                   # Go 3 Pod 共通 chart
│   ├── tier1-rust-service/             # Rust 3 Pod ループ
│   ├── tier2-go-service/               # 汎用 Go テンプレ
│   ├── tier2-dotnet-service/           # 汎用 .NET テンプレ
│   ├── tier3-bff/                      # Go BFF テンプレ
│   └── tier3-web-app/                  # nginx + SPA + reverse proxy
├── kustomize/                          # 環境別 overlay
│   ├── base/                           # 共通 Namespace + label
│   └── overlays/{dev,staging,prod}/    # 6 chart × 3 環境 = 18 values overlay
├── rollouts/                           # Argo Rollouts 関連
│   ├── canary-strategies/              # canary 25→50→100 段階戦略
│   ├── analysis/                       # ★ 共通 ClusterAnalysisTemplate 5 本（IMP-REL-AT-040〜044）
│   ├── analysis-templates/             # サービス固有 AnalysisTemplate 例
│   └── experiments/                    # 採用後の運用拡大時 で追加
├── image-updater/                      # Argo CD Image Updater 設定
└── opentofu/                           # OpenTofu モジュール（採用後の運用拡大時）
    ├── environments/{dev,staging,prod}/
    └── modules/{vpn-gateway,dns,baremetal-k8s}/
```

## Sync Wave の設計

Argo CD の `argocd.argoproj.io/sync-wave` annotation で起動順序を制御する。
詳細は [`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md`](../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md)。

| Wave | 対象 | 前提条件 |
|------|------|----------|
| -10 | Operator + CRD（CNPG / Strimzi / Istio CNI / Dapr Operator / cert-manager / SPIRE） | （クラスタ初期化完了） |
| -5 | CR 宣言（Dapr Components / Istio AuthorizationPolicy / KRaft Cluster CR） | Operator が CRD 認識済み |
| 0 | data backend（CNPG Cluster / Kafka Cluster / MinIO / Valkey）+ LGTM | CRD 認識済み |
| 10 | tier1（Go ファサード 3 Pod + Rust 3 Pod） | data backend Ready |
| 20 | tier2（.NET / Go ドメインサービス） | tier1 公開 API Ready |
| 30 | tier3（Web / BFF / Native は image 配布外 / Legacy wrap） | tier2 ドメインサービス Ready |
| 40〜45 | ops（Backstage / chaos / runbook） | アプリ Pod 起動完了 |

## デプロイ手順（Argo CD 初期化）

```sh
# 1. Argo CD を install（infra/ から）
kubectl apply -f infra/k8s/namespaces/argocd.yaml
helm install argocd argo/argo-cd --namespace argocd --create-namespace

# 2. AppProject 群を先に apply（app-of-apps の前提）
kubectl apply -f deploy/apps/projects/

# 3. RBAC ConfigMap を apply
kubectl apply -f deploy/apps/projects/rbac.yaml

# 4. ルート Application を apply（以後は ApplicationSet が自動展開）
kubectl apply -f deploy/apps/app-of-apps.yaml
```

以降の更新は **Git PR のみ**。`kubectl apply` を二度と打たない（GitOps 原則）。

## 関連設計

- [ADR-CICD-001](../docs/02_構想設計/adr/ADR-CICD-001-gitops.md) — GitOps（Argo CD）採用
- [ADR-CICD-002](../docs/02_構想設計/adr/ADR-CICD-002-argo-rollouts.md) — Argo Rollouts（progressive delivery）
- [ADR-REL-001](../docs/02_構想設計/adr/ADR-REL-001-progressive-delivery-required.md) — PD 必須化
- [IMP-DIR-OPS-092](../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md) — ApplicationSet 配置
- [`docs/05_実装/70_リリース設計/40_AnalysisTemplate/`](../docs/05_実装/70_リリース設計/40_AnalysisTemplate/) — 共通 ClusterAnalysisTemplate 5 本
