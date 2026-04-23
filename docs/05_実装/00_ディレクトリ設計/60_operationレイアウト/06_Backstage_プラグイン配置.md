# 06. Backstage プラグイン配置

本ファイルは Backstage プラグイン実装の配置を確定する。Backstage は開発者ポータル（Developer Portal）として、k1s0 の Software Catalog・Tech Docs・Scaffolder を統合提供する。

## Backstage の役割

2 名運用前提で 10 個以上のサービスが増える状況では、「どのサービスが誰の管轄か」「docs はどこか」「新サービス起動手順は何か」が瞬時にわからないと運用が崩壊する。Backstage は以下を集約する。

- **Software Catalog**: 全サービスのメタデータ・ownership・依存関係
- **TechDocs**: Markdown を MkDocs で静的サイト化
- **Scaffolder**: 雛形 CLI の Web UI 版（tier2 新サービス作成等）
- **API ドキュメント**: OpenAPI / Protobuf 自動表示

## 配置先

Backstage 本体は `src/platform/backstage-plugins/` に配置。`src/platform/` は ADR-DIR-001 由来の「雛形・ツール・内製プラグイン」を集約する場所。

```
src/platform/
├── README.md
├── cli/                            # k1s0 CLI（Rust）
├── analyzer/                       # Roslyn / go-linter plugin
└── backstage-plugins/              # Backstage 内製プラグイン
    ├── README.md
    ├── package.json                # pnpm workspace
    ├── pnpm-workspace.yaml
    ├── plugins/
    │   ├── k1s0-catalog-processor/ # Software Catalog custom processor
    │   ├── k1s0-scaffolder/        # tier2 新サービス scaffold action
    │   ├── k1s0-techdocs/          # TechDocs カスタマイズ
    │   ├── k1s0-observability/     # Grafana / Mimir 統合
    │   └── k1s0-tier-view/         # tier1/2/3 ツリー表示
    ├── backstage-app/              # Backstage 本体（app / backend）
    │   ├── packages/
    │   │   ├── app/                # React frontend
    │   │   └── backend/            # Node.js backend
    │   └── Dockerfile
    └── catalog-entities/           # catalog-info.yaml サンプル
        └── tier1-facade-service-invoke.yaml
```

## Backstage のデプロイ先

Backstage は `infra/observability/` 近辺にはせず、独立した `k1s0-platform-tools` namespace にデプロイする。namespace 名が ArgoCD AppProject 名（`k1s0-platform`）と混同しないよう `-tools` を suffix に付与する（ADR-DIR-002 の infra 分離方針に沿い、namespace 定義は `infra/k8s/namespaces/k1s0-platform-tools.yaml` で管理）。配信定義は `deploy/apps/application-sets/ops.yaml` で管理。

### namespace 定義の配置

Backstage のデプロイに必要な namespace は以下の順で bootstrap される。

1. `infra/k8s/namespaces/k1s0-platform-tools.yaml` — Backstage 本体・関連 Pod
2. `infra/security/*` で Keycloak / SPIRE の SSO 連携リソースを `k1s0-platform-tools` namespace に配置
3. `deploy/apps/application-sets/ops.yaml` で Helm chart を sync（Sync Wave 40、tier1〜tier3 起動後）

namespace 名と AppProject 名の使い分けは以下:

| 種別 | 名前 | 用途 |
|---|---|---|
| Kubernetes Namespace | `k1s0-platform-tools` | Backstage Pod / ConfigMap / Service の物理配置先 |
| ArgoCD AppProject | `k1s0-platform` | 本プロジェクト全 Application の RBAC 境界 |

`50_infraレイアウト/02_k8sブートストラップ.md` の namespace 一覧に `k1s0-platform-tools` を追加登録する。

## catalog-info.yaml の配置規則

各サービスは自らのディレクトリに `catalog-info.yaml` を置く。

```yaml
# src/tier1/go/facade/service-invoke/catalog-info.yaml
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: tier1-facade-service-invoke
  title: tier1 Service Invoke Facade
  description: Dapr Service Invoke API の Go ファサード
  tags:
    - tier1
    - go
    - dapr
  annotations:
    github.com/project-slug: k1s0/k1s0
    backstage.io/techdocs-ref: dir:.
spec:
  type: service
  lifecycle: production
  owner: group:tier1-go
  system: k1s0-tier1
  dependsOn:
    - component:tier1-rust-crypto
  providesApis:
    - tier1-service-invoke-api
```

Backstage Catalog Processor が GitHub から定期的に `catalog-info.yaml` を収集し、Software Catalog に反映する。

## Scaffolder の役割

「新 tier2 サービス作成」「新 tier3 web アプリ作成」等を Web UI 化。`src/platform/cli/` の CLI 実装を呼び出すラッパー。

```yaml
# backstage-plugins/plugins/k1s0-scaffolder/templates/tier2-dotnet-service.yaml
apiVersion: scaffolder.backstage.io/v1beta3
kind: Template
metadata:
  name: tier2-dotnet-service
  title: tier2 .NET サービス作成
spec:
  parameters:
    - title: サービス情報
      properties:
        serviceName:
          type: string
          pattern: '^[a-z][a-z0-9-]+$'
        ownerTeam:
          type: string
          enum: [tier2-dev, tier2-fintech, tier2-payroll]
  steps:
    - id: run-cli
      name: k1s0 CLI scaffold
      action: k1s0:tier2:dotnet:scaffold
      input:
        serviceName: ${{ parameters.serviceName }}
        ownerTeam: ${{ parameters.ownerTeam }}
    - id: publish
      action: publish:github:pull-request
      input:
        repoUrl: github.com?repo=k1s0&owner=k1s0
        branchName: scaffold/${{ parameters.serviceName }}
        title: 'scaffold: tier2 ${{ parameters.serviceName }}'
```

## Phase 導入タイミング

| Phase | 内容 |
|---|---|
| Phase 0 | 構造のみ |
| Phase 1a | Software Catalog + TechDocs 最小（手動 catalog-info） |
| Phase 1b | Scaffolder（tier2 / tier3 新サービス作成） |
| Phase 1c | Observability 統合、API ドキュメント自動連動 |
| Phase 2 | 独自プラグイン本格展開 |

## 対応 IMP-DIR ID

- IMP-DIR-OPS-096（Backstage プラグイン配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-DEVEX-001（Backstage 採用）
- DX-GP-\* / DX-CICD-\*
