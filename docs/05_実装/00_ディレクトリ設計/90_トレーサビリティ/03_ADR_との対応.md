# 03. ADR との対応

本ファイルは `IMP-DIR-*` と ADR（Architecture Decision Record）の対応を明示する。

## 本プランで新規起票した 3 本

### ADR-DIR-001: contracts 昇格

`src/tier1/contracts/` を `src/contracts/` に昇格。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-T1-022（contracts 配置） | 直接（主論点） |
| IMP-DIR-T1-025（SDK 配置） | 直接（契約→SDK 生成フロー） |
| IMP-DIR-T1-026（生成コードの扱い） | 直接 |
| IMP-DIR-ROOT-009（src/ 層別分割） | 間接（src 直下に contracts が昇格） |
| IMP-DIR-COMM-116（codegen 配置） | 間接（buf 実行元） |

### ADR-DIR-002: infra 分離

`src/tier1/infra/` を廃止し、`infra/`（素構成）/ `deploy/`（GitOps）/ `ops/`（Runbook）に 3 階層分離。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-INFRA-071〜078（infra 全般） | 直接（主論点） |
| IMP-DIR-OPS-091〜097（deploy + ops 全般） | 直接 |
| IMP-DIR-ROOT-010（横断ディレクトリ） | 間接（ルート昇格） |
| IMP-DIR-ROOT-012（依存方向ルール） | 間接（tier1 → infra の禁止方向確認） |

### ADR-DIR-003: sparse-checkout cone mode 採用

Git sparse-checkout cone mode + partial clone + sparse index を標準運用として推奨。9 役別 cone 定義を規定。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-SPARSE-126〜132（スパースチェックアウト全般） | 直接（主論点） |
| IMP-DIR-ROOT-007（cone 統合原則） | 直接 |
| IMP-DIR-COMM-111（tools 配置）| 間接（checkout-role.sh 等） |
| IMP-DIR-COMM-115（devcontainer 配置） | 間接（役割別 Dev Container） |

## 既存 ADR との対応

### ADR-TIER1-003（内部言語不可視）

tier2 / tier3 から tier1 の内部言語（Rust / Go 判別）は不可視。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-T1-025（SDK 配置） | 直接（SDK が言語を隠蔽） |
| IMP-DIR-T2-041（tier2 全体配置） | 間接（tier2 から見た API の受け取り方） |
| IMP-DIR-T3-056（tier3 全体配置） | 間接 |
| IMP-DIR-ROOT-012（依存方向ルール） | 直接 |

### ADR-MIG-001（.NET Framework sidecar）

既存 .NET Framework 資産を Dapr sidecar パターンで k1s0 基盤に接続する段階的移行戦略。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-T3-060（レガシーラップ配置） | 直接（主実装先） |

### ADR-MIG-002（API Gateway）

既存システムの API Gateway 経由での段階的移行。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-INFRA-073（サービスメッシュ配置） | 直接（Envoy Gateway） |
| IMP-DIR-T3-060（レガシーラップ配置） | 間接 |

### ADR-CNCF-001〜005（CNCF OSS 選定）

CNI / Longhorn / Istio / Envoy / Dapr の採用決定。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-INFRA-072〜074 | 直接 |

### ADR-DATA-001〜004（データ層 OSS 選定）

CloudNativePG / Kafka / Valkey / MinIO の採用決定。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-INFRA-075 | 直接 |

### ADR-SEC-001〜005（セキュリティ OSS 選定）

Keycloak / OpenBao / SPIRE / cert-manager / Kyverno の採用決定。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-INFRA-076 | 直接 |

### ADR-OBS-001〜003（観測性 OSS 選定）

LGTM / OpenTelemetry / Pyroscope の採用決定。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-INFRA-077 | 直接 |

### ADR-CICD-001〜006（CI/CD 関連）

GitOps / ArgoCD / Argo Rollouts / Helm / Kustomize / OpenTofu の採用決定。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-OPS-091〜097 | 直接 |

### ADR-DEVEX-001〜004（開発者体験）

Backstage / Dev Container / テスト戦略 / Golden Path の採用決定。

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-OPS-096（Backstage） | 直接 |
| IMP-DIR-COMM-111〜116 | 直接 |

### ADR-GOV-001（OSS ライセンス遵守ポリシー）

| IMP-DIR ID | 関連度 |
|---|---|
| IMP-DIR-COMM-114（third_party 配置） | 直接 |

## 未起票の ADR（Phase 1c 以降判定）

- ADR-DIR-004: Git LFS ポリシー
- ADR-DIR-005: CODEOWNERS 構造（現時点は `06_CODEOWNERSマトリクス設計.md` に記述のみ）
- ADR-DIR-006: Bazel / Buck2 / Nx / Turborepo 等のビルドツール導入

これらは Phase 1c で判定する。本プランでは採番しない。
