# deploy/kustomize — 環境別 Kustomize / Helm values overlay

Argo CD ApplicationSet が環境別（dev / staging / prod）に Helm chart を配備する際の
**values overlay** と **base 共通リソース** を本ディレクトリに集約する。

## 配置

```
deploy/kustomize/
├── README.md                              # 本ファイル
├── base/                                  # 全環境共通の追加リソース
│   ├── kustomization.yaml
│   └── README.md
└── overlays/
    ├── dev/                               # 開発環境（kind / 単一 cluster）
    │   ├── kustomization.yaml
    │   ├── tier1-facade-values.yaml
    │   ├── tier1-rust-service-values.yaml
    │   ├── tier2-go-service-values.yaml
    │   ├── tier2-dotnet-service-values.yaml
    │   ├── tier3-bff-values.yaml
    │   └── tier3-web-app-values.yaml
    ├── staging/                           # ステージング環境
    │   └── （同上 6 values 構成）
    └── prod/                              # 本番環境（HA + 監視 + Rollouts）
        └── （同上 6 values 構成）
```

## ApplicationSet との連携

`deploy/apps/application-sets/<chart>.yaml` は本 overlay を `helm.valueFiles` の
multi-source（`$values/deploy/kustomize/overlays/<env>/<chart>-values.yaml`）として
読み込む。tier1-facade ApplicationSet がリファレンス実装。

## 環境別の差分原則

| 観点 | dev | staging | prod |
|---|---|---|---|
| replica | 1 | 2 | 3 以上（HA） |
| log level | debug | info | warn |
| resources requests | 最小 | 中 | 実測ベース |
| HPA | 無効 | 有効（緩い） | 有効（実測） |
| TLS | 任意 | 強制 | 強制（mTLS STRICT） |
| Image tag | `latest` | release candidate | semver pinning |
| 監視 | 任意 | 有効 | 有効（PagerDuty 連携） |
| Argo Rollouts | 任意 | 有効 | 有効（canary 25→50→100） |

## 関連設計

- ADR-CICD-001（Argo CD ApplicationSet）
- ADR-CICD-002（Argo Rollouts canary）
- IMP-REL-* — リリース設計
