# 実装マチュリティ開示（SHIP STATUS）

本ファイルは k1s0 リポジトリの**実装側の現状**と**docs（設計側）の規定**との
ギャップを領域別に開示する。docs（約 658 ファイル / 5.3 万行）が記述する全体像と、
実際に `git clone` 直後にビルド・起動できる範囲との差分は大きい。
OSS 採用検討者が誤った前提で評価しないよう、本ファイルは現状を**段階表 3 ランク**で
正直に記述する。

## 想定読者

- k1s0 を採用検討中のアーキテクト・SRE
- リポジトリを clone してから「何が動き、何が動かないか」を最短で知りたい開発者
- docs のリリース範囲表（README「リリース範囲」節）の実装側の裏付けを確認したい人

## 前提となる用語（docs 正典）

docs では構成要素を以下 3 段階で論じている。本ファイルもこの語彙に揃える。

| 段階 | 意味 | docs 内典型例 |
|---|---|---|
| **リリース時点** | OSS 公開（v0 タグ）時点で同梱されるべき範囲 | 12 公開 API contracts / 主要 ADR / 採用初期スタック |
| **採用初期** | 採用組織が POC〜本番初期投入する段階で完成すべき範囲 | tier1 全 Pod ハンドラ / Helm chart / Argo CD ApplicationSet |
| **採用後の運用拡大時** | 複数チーム導入・マルチクラスタ化フェーズで導入される拡張 | OpenTofu / マルチクラスタフェデレーション / 高度な observability |

本ファイルが扱うマチュリティの 3 ランクは以下の通り。

| ランク | 意味 |
|---|---|
| **同梱済** | 実コードが存在し、ビルド・起動・テストが走る状態 |
| **雛形あり** | ディレクトリ構造・主要ファイル・README は存在するが、ロジック実装は未完または最小骨格 |
| **設計のみ** | docs に詳細設計があるが、対応する実装側ディレクトリは空または `.gitkeep` のみ |

## 領域別マチュリティ表

### tier1（公開 12 API + Dapr ファサード + Rust コア）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/contracts/tier1/` proto 12 サービス | 12 API（state / pubsub / serviceinvoke / secrets / binding / workflow / log / telemetry / decision / audit / feature / pii） | **同梱済** | 14 proto / 47 RPC（公開 43 + health 2 + admin 2）配置済。`buf lint` / `buf format` 通過、4 SDK 再生成済。共通型は `common/v1/common.proto`（TenantContext / ErrorDetail / K1s0ErrorCategory）に集約 |
| `src/contracts/internal/` proto | tier1 内部 gRPC（Go ↔ Rust core、ADR-TIER1-002 / ADR-TIER1-003） | **同梱済** | 4 proto / 3 service / 8 RPC 配置済（DS-SW-IIF-004〜029）。`buf generate --template buf.gen.internal.yaml` で Go と Rust に再生成 |
| `src/tier1/go/cmd/{state,secret,workflow}/` | Go 側 3 Pod（DS-SW-COMP-005/006/010） | **同梱済**（in-memory backend） | gRPC server bootstrap + 全 9 公開 API handler 登録完了（t1-state: 7 = State / PubSub / ServiceInvoke / Binding / Feature / Log / Telemetry、t1-secret: 1、t1-workflow: 1）。`DAPR_GRPC_ENDPOINT` / `OPENBAO_ADDR` / `TEMPORAL_HOSTPORT` 未設定時は in-memory backend で fallback し、State Get/Set/Delete/Bulk*/Transact、Secrets Get/BulkGet/Rotate/GetDynamic、Workflow Start/Signal/Query/Cancel/Terminate/GetStatus が実値を返す。Log / Telemetry handler は stdout JSON Lines emitter で稼働（OTel Collector 結線は post-MVP）。**HealthService（Liveness / Readiness 2 RPC）も全 3 Pod で登録済**（`internal/health/` に共通実装、Pod ごとの依存先 probe を結線: state→dapr / secret→openbao / workflow→temporal+dapr-workflow）。全 handler で **NFR-E-AC-003 tenant_id 越境防止** を強制（tenant_id 空は InvalidArgument）。in-memory backend 自体も dev/CI で **テナント間越境を物理的に遮断**（state は metadata.tenantId で partition、daprwf は run.tenantID で resolveLocked、temporal は scopedWorkflowID で WorkflowID prefix）。FeatureAdminService は本リリース時点 未登録（採用初期で追加） |
| `src/tier1/go/internal/common/` | 共通 runtime（gRPC bootstrap / config / retry / timeout） | **同梱済** | runtime / config / retry / timeout の 4 ユーティリティとテストが存在 |
| `src/tier1/go/internal/otel/` | OTel 初期化 | **雛形あり** | 1 ファイルの最小骨格 |
| `src/tier1/rust/crates/{decision, audit, pii}` | Rust 側 3 Pod（DS-SW-COMP-008/007/009） | **同梱済**（in-memory backend） | 3 crate に gRPC handler 完全実装。decision: ZEN Engine 0.55+ JDM 評価 / 評価トレース / registry。audit: SHA-256 hash chain WORM in-memory store + Record / Query / Export(streaming) / VerifyChain（FR-T1-AUDIT-002 / NFR-H-INT-001）。pii: classify + mask（email / credit card / IPv4 / mynumber / phone）。全件 tonic server + reflection + graceful shutdown + テナント分離。Postgres / 外部 backend は post-MVP |
| `src/tier1/rust/crates/{common, otel-util, policy, proto, proto-gen}` | 共通 crate / proto stub | **雛形あり** | proto-gen crate 配置済、common / otel-util / policy / proto は `workspace.exclude` に置き plan 04-08 で順次合流予定 |
| Dockerfile（distroless / nonroot / multi-stage） | 3 Pod 各 1 Dockerfile | **同梱済** | `Dockerfile.{state,secret,workflow}` は完成 |

### contracts と SDK 生成

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `buf.gen.yaml` | 4 言語生成設定 | **同梱済** | tier1 公開と internal の 2 module 構成 |
| `src/sdk/go/generated/` | Go gRPC stub | **同梱済** | 14 proto 分の正式 RPC 群を生成済 |
| `src/sdk/dotnet/generated/` | .NET stub | **同梱済** | 14 proto 分を生成済 |
| `src/sdk/rust/generated/` | Rust prost / tonic stub | **同梱済** | 14 proto 分を生成済 |
| `src/sdk/typescript/generated/` | TS protobuf-es / connect-es stub | **同梱済** | 14 proto 分を生成済 |
| 高水準 SDK（`k1s0.State.Save(...)` 等の動詞統一） | docs 規定の 4 言語動詞 | **同梱済**（14 service 全件） | 4 言語すべてに Client + 動詞統一 facade を 14 service 全件（公開 12 + Admin 2）で実装。Stream RPC（InvokeStream / PubSub.Subscribe）も 4 言語で同梱 |

### tier2（C# / Go ドメイン共通）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/tier2/templates/` + `src/tier3/templates/` | k1s0-scaffold が参照する Backstage Software Template v1beta3（IMP-CODEGEN-SCF-031〜034） | **同梱済** | 4 テンプレ配置済 |
| `src/tier2/dotnet/services/{ApprovalFlow, InvoiceGenerator, TaxCalculator}` | tier2 完動例 | **同梱済** | 3 サービス DDD レイヤード（Domain / Application / Infrastructure / Api）+ xUnit ドメイン単体 + ArchitectureTests（NetArchTest）+ Dockerfile + catalog-info |
| `src/tier2/go/services/{notification-hub, stock-reconciler}` | tier2 Go 完動例 | **同梱済** | 2 サービス cmd + internal/{api,config}/ + Dockerfile + catalog-info |

### tier3（Web / Native / BFF / Legacy）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/tier3/web/apps/{portal, admin, docs-site}` | React + Vite + pnpm | **同梱済** | 3 SPA（main.tsx / App.tsx / pages 2 件 / vitest 単体）+ Dockerfile（nginx + SPA fallback）+ catalog-info |
| `src/tier3/web/packages/{ui, api-client, i18n, config}` | 共通パッケージ | **同梱済** | 4 package（ui: Button/Card/Spinner、api-client、i18n: ja/en、config）+ vitest 単体テスト |
| `src/tier3/bff/cmd/{portal-bff, admin-bff}` | Go BFF | **同梱済** | 2 BFF cmd + GraphQL（schema.graphql）+ REST + auth/middleware + k1s0client + shared/{errors,otel} + Dockerfile.{portal,admin} |
| `src/tier3/native/apps/{Hub, Admin}` | .NET MAUI | **同梱済** | 2 アプリ MAUI（App.xaml + AppShell + Pages + ViewModels + Services + Platforms/{Android,iOS}）+ shared/K1s0.Native.Shared + Native.sln |
| `src/tier3/legacy-wrap/sidecars/K1s0.Legacy.Sidecar` | .NET Framework サイドカー | **同梱済** | ASP.NET Web API サイドカー（Global.asax + WebApiConfig + DaprConfig + K1s0BridgeController + DaprClientAdapter + StateValue + Web.config + packages.config + Dockerfile.windows）+ migration-guide 3 ステップ |

### platform（CLI / Backstage プラグイン / Analyzer）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/platform/scaffold/` | k1s0-scaffold CLI（Rust 実装、IMP-CODEGEN-SCF-030） | **同梱済** | Rust crate（edition 2024）+ 5 ソース（main.rs / lib.rs / template.rs / engine.rs / error.rs）+ README |
| `src/platform/analyzer/` | .NET 依存方向 Roslyn Analyzer（IMP-DIR-ROOT-002）| **同梱済** | 4 診断 ID（K1S0DEPDIR0001〜0004）すべて Severity=Error。3 csproj + xUnit Tests + sln |
| `src/platform/backstage-plugins/` | Backstage 開発者ポータル plugin（ADR-DEVEX-002） | **雛形あり** | 2 plugin（k1s0-catalog / k1s0-scaffolder）の skeleton |

### infra（k8s / mesh / data / observability / security）

| 領域 | docs 規定（リリース必須） | 実装ランク | 備考 |
|---|---|---|---|
| `infra/k8s/{bootstrap, namespaces, networking, storage}` | kubeadm HA + Calico/MetalLB | **雛形あり** | bootstrap: Cluster API + KubeadmControlPlane HA 3。namespaces: 17 layer に Pod Security Standards label + Istio Ambient ラベル。networking: Calico VXLAN + MetalLB。storage: 4 種 StorageClass |
| `infra/mesh/istio-ambient/` | ADR-0001 / ADR-MIG-002 | **雛形あり** | profile ambient + istiod HA 3 replica + HPA + OTel tracing 連携 + ztunnel 設定 + mTLS STRICT |
| `infra/mesh/envoy-gateway/` | ADR-CNCF-004 / IMP-DIR-INFRA-073 | **雛形あり** | Envoy Gateway controller HA 3 + GatewayClass + Gateway（public/internal）+ HTTPRoute（tier1-api / tier3-web / redirect）+ OTel tracing + Prometheus ServiceMonitor |
| `infra/dapr/control-plane/` | Dapr operator | **雛形あり** | HA 全 control-plane component（operator/placement/sentry/sidecar-injector/scheduler）3 replica + mTLS + Prometheus + Raft consensus |
| `infra/dapr/components/` + `infra/dapr/subscriptions/` | Dapr Component CRD（IMP-DIR-INFRA-074） | **雛形あり** | 7 Component（state/postgres / state/redis-cache / pubsub/kafka / secrets/vault / binding/s3-inbound / binding/smtp-outbound / configuration/default）+ 2 Subscription（audit-pii / feature） |
| `infra/data/{cloudnativepg, kafka, minio, valkey}` | ADR-DATA-001/002/003/004 | **雛形あり** | 4 backend を production-grade defaults（HA / 監視 / バックアップ）で正規化済 |
| `infra/security/{cert-manager, keycloak, openbao, spire, kyverno}` | ADR-SEC-001/002/003 / ADR-POL-001 | **雛形あり** | 5 component すべて HA 3 + 認証統合 + ServiceMonitor |
| `infra/observability/{loki, tempo, mimir, grafana, otel-collector, pyroscope}` | ADR-OBS-001/002 | **雛形あり** | LGTM 6 component を production-grade で正規化済 |
| `infra/scaling/keda/` | KEDA | **雛形あり** | operator HA 2 + metrics-apiserver 2 + admission webhook + ServiceMonitor |
| `infra/feature-management/flagd/` | ADR-FM-001 | **雛形あり** | flagd HA 3 replica + ConfigMap baseline flag + Service + ServiceMonitor |
| `infra/environments/{dev, staging, prod}` | 環境別 overlay（IMP-DIR-INFRA-078） | **雛形あり** | 3 環境 overlay（kustomization + patches + values + secrets/.gitkeep）+ README |

### deploy（GitOps / Helm / Kustomize / OpenTofu）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `deploy/README.md` | GitOps 配信定義の入口（IMP-DIR-COMM-110） | **同梱済** | deploy 配下の構造・Sync Wave・初期化手順を明記 |
| `deploy/apps/app-of-apps.yaml` | App-of-Apps ルート（IMP-DIR-OPS-092） | **同梱済** | k1s0-platform AppProject 所属の Argo CD ルート Application |
| `deploy/apps/application-sets/{infra,ops,tier1-facade,tier1-rust-service,tier2-dotnet-service,tier2-go-service,tier3-bff,tier3-web-app}.yaml` | カテゴリ別 ApplicationSet | **同梱済** | 8 ApplicationSet 配置完了。infra（Wave -10、list-generator）/ ops（Wave 40〜45）/ 6 サービス系（Wave 10〜30、image-updater annotation 付き） |
| `deploy/apps/projects/{k1s0-platform,k1s0-tier1,k1s0-tier2,k1s0-tier3,rbac}.yaml` | AppProject + RBAC | **同梱済** | 4 AppProject + Argo CD グローバル RBAC ConfigMap（OIDC 連携） |
| `deploy/charts/{tier1-facade, tier1-rust-service, tier2-go-service, tier2-dotnet-service, tier3-bff, tier3-web-app, predeploy-hooks}` | Helm chart | **雛形あり** | 7 chart 全て配置完了。`helm lint` 通過、`helm template` 描画 OK。`predeploy-hooks` は Argo CD PreSync Hook で Postgres / Kafka / Valkey / MinIO の readiness を polling 検証する Job 4 種を提供（Sync Wave -1）。設計: `docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md` |
| `deploy/rollouts/canary-strategies/` | Argo Rollouts canary 戦略 | **雛形あり** | canary 25→50→100% の 3 段階戦略テンプレート |
| `deploy/rollouts/analysis/` | 共通 ClusterAnalysisTemplate 5 本（IMP-REL-AT-040〜049） | **同梱済** | at-common-{error-rate,latency-p99,cpu,dependency-down,error-budget-burn}.yaml の 5 本（baseline 2σ / SLO 連動 / CPU 80% / 依存断短絡 / EB burn 2x）+ README |
| `deploy/rollouts/analysis-templates/` | サービス固有 AnalysisTemplate 例 | **雛形あり** | error-rate.yaml / latency-p99.yaml の 2 例 |
| `deploy/kustomize/{base, overlays/*}` | Kustomize | **雛形あり** | base + overlays/{dev,staging,prod}/ に 6 chart × 3 環境 = 18 values overlay |
| `deploy/opentofu/{environments, modules}` | OpenTofu（採用後の運用拡大時） | **設計のみ** | `.gitkeep` のみ（採用後の運用拡大時 で展開、現段階では意図的に空） |
| `deploy/image-updater/` | Argo CD Image Updater | **雛形あり** | argocd-image-updater Helm values + registries.conf ConfigMap + application-annotations.md |

### tools / tests / examples

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `tools/local-stack/` | kind ベース本番再現スタック（IMP-DEV-POL-006） | **同梱済** | up.sh / down.sh / status.sh / kind-cluster.yaml / 17 レイヤ namespace yaml |
| `tools/local-stack/manifests/{20..95}_*/` | 各レイヤの Helm values / Kustomize | **同梱済** | 17 レイヤ全てに values.yaml または manifest.yaml 配置済 |
| `tools/devcontainer/` | 10 役 Dev Container プロファイル | **雛形あり** | postCreate.sh / doctor.sh / README は存在 |
| `tools/sparse/` | sparse-checkout 10 役 cone 定義 | **雛形あり** | checkout-role.sh / verify.sh / README は存在 |
| `tools/codegen/` | buf / openapi / grpc-docs 生成ラッパ | **同梱済** | 3 ラッパ script（buf/run.sh / openapi/run.sh / grpc-docs/run.sh） |
| `tools/git-hooks/` | 自作 pre-commit hook | **同梱済** | japanese-header-guard.py / file-length-guard.py / drawio-svg-staleness.sh / link-check-wrapper.py |
| `tools/_link_check.py` / `_link_fix.py` / `_export_svg.py` | docs 横断ツール | **同梱済** | docs リンク検査・drawio SVG export |
| `tools/ci/go-dep-check/` + `tools/ci/rust-dep-check/` | 依存方向 linter（IMP-DIR-ROOT-002） | **同梱済** | Go / Rust 両側に独立 go.mod、`tier3 → tier2 → sdk → tier1` 一方向ルールを `import` / `path` 依存レベルで強制 |
| `tests/` | e2e / contract / integration / fuzz / golden / fixtures | **雛形あり** | 6 カテゴリすべてに README + 動作可能な最小骨格 + CI hook 連携の入口 |
| `examples/` | Golden Path 7 プロジェクト | **雛形あり** | 7 種すべてが build 可能な完動例。各 example に Dockerfile + catalog-info.yaml + 週次 E2E workflow |

### ルート README 群（ドキュメント近接配置方針 / IMP-DIR-COMM-110）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/README.md` | src 配下の入口 | **同梱済** | 配置構造・依存方向・コーディング規約・サブ設計リンク |
| `src/contracts/README.md` | contracts の入口 | **同梱済** | 公開 12 API + 内部 gRPC 一覧、生成パス、buf ゲート |
| `src/tier1/README.md` | tier1 の入口 | **同梱済** | 6 Pod 構成（DS-SW-COMP-005〜010）、ローカル起動手順 |
| `src/tier2/README.md` | tier2 の入口 | **同梱済** | DDD レイヤ構造、言語選択指針、tier1 SDK 使用例 |
| `src/tier3/README.md` | tier3 の入口 | **同梱済** | web / bff / native / legacy-wrap 配置、BFF パターン、k1s0.io annotation |
| `src/platform/README.md` | platform の入口 | **同梱済** | scaffold CLI / analyzer / Backstage plugins |
| `infra/README.md` | infra の入口 | **同梱済** | k8s / mesh / dapr / data / security / observability / scaling / feature-management / environments の俯瞰 |
| `deploy/README.md` | deploy の入口 | **同梱済** | apps / charts / kustomize / rollouts / opentofu の俯瞰、Sync Wave 設計、Argo CD 初期化手順 |
| `ops/README.md` | ops の入口 | **同梱済** | runbooks / chaos / dr / oncall / load の俯瞰、Runbook 5 段構成 |
| `tools/README.md` | tools の入口 | **同梱済** | local-stack / devcontainer / codegen / sparse / git-hooks / ci の俯瞰、CI ゲート連携 |
| `tests/README.md` | tests の入口 | **同梱済** | （既存） |
| `examples/README.md` | examples の入口 | **同梱済** | （既存） |

### CI / CD / GitOps

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `.github/workflows/pr.yml` | path-filter で 11 軸検出 + reusable workflow + ci-overall 集約 | **同梱済** | path-filter / reusable 4 本 / commitlint まで構成済 |
| `.github/workflows/_reusable-{build,test,lint,precommit,push}.yml` | 言語別 reusable workflow | **同梱済** | 4 言語分岐 + SBOM(syft) 生成設計済 |
| `.github/workflows/labels-sync.yml` / `renovate.yml` | リポジトリ運用 | **同梱済** | |
| `.github/CODEOWNERS` / `labels.yml` / `repo-settings.md` | リポジトリ設定の正典化 | **同梱済** | |

### docs / ADR

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `docs/01_企画/` | 採用検討向け企画資料 | **同梱済** | |
| `docs/02_構想設計/adr/` | 36 ADR | **同梱済** | 全 ADR 1 行索引 + 詳細索引の二重管理 |
| `docs/02_構想設計/{01_アーキテクチャ, 02_tier1設計, 03_技術選定, 04_CICDと配信, 05_法務とコンプライアンス}` | 構想設計 | **同梱済** | |
| `docs/03_要件定義/` | IPA 共通フレーム 2013 準拠 10 カテゴリ | **同梱済** | FR-T1 / NFR / BR / OR / DX / BC / RISK 体系 |
| `docs/04_概要設計/` | DS-* 12 カテゴリ | **同梱済** | DS-SYS / DS-SW / DS-CF / DS-CTRL / DS-NFR / DS-OPS / DS-MIG / DS-DEVX / DS-BUS |
| `docs/05_実装/` | IMP-* 13 章 | **同梱済** | IMP-DIR / IMP-BUILD / IMP-CODEGEN / IMP-CI / IMP-DEP / IMP-DEV / IMP-OBS / IMP-REL / IMP-SUP / IMP-SEC / IMP-POL / IMP-DX / IMP-TRACE |
| `docs/40_運用ライフサイクル/` | Runbook（タイプ C 5 段構成） | **同梱済** | |
| `docs/90_knowledge/` | 技術学習資料 | **同梱済** | |
| `docs/INDEX.md` | 階層ナビゲーション索引 | **同梱済** | |

## 「docs から逸脱しない」ことの保証

本リポジトリの実装作業は、以下のメカニズムで docs 正典との整合性を保つ。

1. **ADR が先行**: 構造を変える PR は `docs/02_構想設計/adr/` に新 ADR を起票してから着手する（CONTRIBUTING.md 規定）
2. **ID 体系**: 要件 ID（FR-T1-*/ NFR-* / BR-*/ etc.）と設計 ID（DS-* / IMP-*）は実装側コミットメッセージにも追跡される（IMP-TRACE-*）
3. **トレーサビリティ索引**: `docs/03_要件定義/80_トレーサビリティ/` と `docs/04_概要設計/80_トレーサビリティ/` で要件 → 設計 → ADR の対応が網羅される
4. **CI ゲート**: `buf lint` / `buf breaking` / 内製 analyzer / pre-commit が逸脱を物理的に遮断する

## 採用検討者へのガイダンス

- **「docs を信じて全部動く」前提では採用しない**こと。本ファイルの「同梱済」のみを動作前提として評価してほしい
- POC 用途では `tools/local-stack/up.sh --role docs-writer`（最小構成）または `--role tier1-rust-dev`（tier1 検証構成）から始めることを推奨する
- 業務適用には「採用初期」段階の実装完了が必要。tier1 全 Pod ハンドラ・Helm chart 全種・examples 完動 4 種が完成してから本番投入を検討すること
- 実装が進むに従い本ファイルは更新される。最新版は main ブランチの本ファイルを参照

## 残存「設計のみ」一覧（リリース時点 v0 対象）

リリース時点（v0.x）では以下のみが「設計のみ」として残る。これらはすべて採用後の運用拡大時段階の対象で、
リリース時点で同梱しないことが docs / ADR 上で確定済み。

| 残存項目 | 段階 | 理由 |
|---|---|---|
| `deploy/opentofu/{environments,modules}/` | 採用後の運用拡大時 | OpenTofu はマルチクラスタ展開時に Terraform 移行先として導入予定。リリース時点で同梱すると未使用コードが採用検討者に誤認される |
| `deploy/rollouts/experiments/` | 採用後の運用拡大時 | Argo Rollouts の Experiment CRD は A/B テスト等の高度な配信機能で、採用初期では使わない |

その他のすべての「リリース時点必須」項目は **同梱済 / 雛形あり** のいずれかに到達している。
雛形ありの項目は採用初期段階で内容実装が進む。

## 次の段階で進めるべき作業（採用初期へのロードマップ）

リリース時点（v0.x）から採用初期段階へ進むために、以下を**docs を逸脱せず**実装する必要がある。
順序は依存方向（contracts → SDK → tier1 → tier2/tier3）と外部評価の費用対効果を考慮した推奨順。

### 1〜8. 完了項目（リリース時点で達成）

contracts / SDK / tier1 / Rust 3 Pod / infra / deploy / examples / SDK 高水準 facade の 8 項目は
リリース時点で完了済み（詳細は本ファイル「領域別マチュリティ表」参照）。

### 9. tier1 ハンドラの実体化（採用初期）

各 RPC の `codes.Unimplemented` を Dapr SDK / OpenBao / Temporal 結線に置換する作業。
依存先（CNPG / Kafka / Valkey / OpenBao / Temporal）の Helm 適用と並行で進める。

### 10. infra Operator 実体化（採用初期）

雛形ありの component を採用組織の k8s クラスタに対して実適用し、`helm install` / `kubectl apply` の
動作確認と SOPS 暗号化 secret 配置を完了する。

### 11. Backstage 実機統合（採用初期）

`src/platform/backstage-plugins/` を採用組織の Backstage バージョンに合わせて
`@backstage/core-plugin-api` 等から import 結線する作業。

### 12. examples ユニット / 統合テスト追加（採用初期）

各 example に対する unit / integration test の雛形を埋める作業。

各タスクは完成のたびに本 SHIP_STATUS.md のマチュリティ表を更新する運用とする。
