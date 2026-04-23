# 01. deploy 配置（GitOps）

本ファイルは `deploy/` 配下の全体構成を確定する。GitOps（ArgoCD）が監視する配信定義の集約場所。`infra/`（素構成）と明確に分離する。

## deploy/ の役割

「どこに」「どう」デプロイするかを記述する層。`infra/` の素構成（何を）、`ops/` の運用（どうオペレートするか）と並び、3 階層を形成する。GitOps 原則により、Git 上の YAML が single source of truth となる。

- ArgoCD Application / ApplicationSet（App-of-Apps パターン）
- 共通 Helm chart（tier1 / tier2 / tier3 アプリのテンプレート）
- Kustomize overlay（環境差分）
- Argo Rollouts AnalysisTemplate（Canary / Blue-Green）
- OpenTofu（ベアメタル / クラウド IaaS プロビジョン）
- Image Updater（Harbor に push された新 image を自動検知）

## レイアウト

```
deploy/
├── README.md
├── apps/                       # ArgoCD Application / ApplicationSet
│   ├── app-of-apps.yaml        # ルート Application（全 ApplicationSet を子に持つ）
│   ├── application-sets/
│   │   ├── infra.yaml          # infra/environments/<env>/ を deploy
│   │   ├── tier1.yaml          # src/tier1/ の各 Pod
│   │   ├── tier2.yaml          # src/tier2/ のサービス
│   │   ├── tier3.yaml          # src/tier3/ の web / bff
│   │   └── ops.yaml            # ops/ の Runbook / chaos manifests
│   └── projects/
│       ├── k1s0-platform.yaml  # AppProject: k1s0 全体
│       └── rbac.yaml
├── charts/                     # 共通 Helm chart
│   ├── tier1-facade/           # Go Dapr facade 共通 chart
│   ├── tier1-rust-service/     # Rust 自作領域共通 chart
│   ├── tier2-dotnet-service/
│   ├── tier2-go-service/
│   ├── tier3-web-app/          # Next.js / Vite 共通 chart
│   └── tier3-bff/
├── kustomize/
│   ├── base/                   # tier1 / tier2 / tier3 共通 base
│   └── overlays/
│       ├── dev/
│       ├── staging/
│       └── prod/
├── rollouts/
│   ├── analysis-templates/
│   │   ├── success-rate-check.yaml
│   │   ├── latency-p99-check.yaml
│   │   └── error-budget-check.yaml
│   ├── canary-strategies/
│   │   ├── 10-25-50-100.yaml    # 段階的 canary
│   │   └── tier1-strict.yaml    # tier1 専用の厳格 canary
│   └── experiments/
│       └── A-B-test.yaml
├── opentofu/
│   ├── modules/
│   │   ├── baremetal-k8s/      # ベアメタルノード プロビジョン
│   │   ├── vpn-gateway/
│   │   └── dns/
│   └── environments/
│       ├── dev/
│       ├── staging/
│       └── prod/
└── image-updater/
    ├── argocd-image-updater-config.yaml
    └── write-back-secret.yaml
```

## 依存方向

- `deploy/apps/application-sets/infra.yaml` → `infra/environments/<env>/`
- `deploy/apps/application-sets/tier1.yaml` → `deploy/charts/tier1-facade/` or `src/tier1/go/deploy/`（chart が同居しない場合）
- `deploy/kustomize/overlays/<env>/` → `deploy/kustomize/base/` + `infra/environments/<env>/`
- `deploy/rollouts/` → ArgoCD Application が `strategy.rollouts` で参照

deploy/ は `src/` を直接コピーしない。Helm chart の values や image タグを参照する形で疎結合を維持する。

## GitOps フロー

1. 開発者が `src/tier1/go/facade/` を変更、PR マージ
2. GitHub Actions（`.github/workflows/ci-tier1-go.yml`）が image ビルド、Harbor に push、tag は `<commit-sha>`
3. ArgoCD Image Updater が Harbor を watch、新 tag を検知して `deploy/apps/.../tier1-facade.yaml` の image tag を書換え（Git commit）
4. ArgoCD Sync が発動、Argo Rollouts で canary deploy
5. `deploy/rollouts/analysis-templates/` が SLI を検証、閾値 OK なら段階的 promotion
6. 失敗時は自動 rollback

## 対応 IMP-DIR ID

- IMP-DIR-OPS-091（deploy 全体配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-CICD-001（GitOps）
- NFR-C-NOP-\* / NFR-D-MIG-\* / DX-CICD-\*
