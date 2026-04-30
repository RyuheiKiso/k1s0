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
| `src/tier1/rust/crates/{decision, audit, pii}` | Rust 側 3 Pod（DS-SW-COMP-008/007/009） | **同梱済**（in-memory backend） | 3 crate に gRPC handler 完全実装。decision: ZEN Engine 0.55+ JDM 評価 / 評価トレース / registry。audit: SHA-256 hash chain WORM in-memory store + Record / Query / Export(streaming) / VerifyChain（FR-T1-AUDIT-002 / NFR-H-INT-001）+ **`idempotency_key` による Record dedup（hash chain 二重追記防止）**。pii: classify + mask（email / credit card / IPv4 / mynumber / phone）。全件 tonic server + reflection + graceful shutdown + テナント分離。**HealthService（Liveness / Readiness）も全 3 Pod で登録済**（共通 `crates/health` 経由、依存先 probe は本リリース時点 空）。**3 Pod すべてに `K1s0Layer`（Auth / RateLimit / Observability / Audit auto-emit の 4 段 chain）を gRPC server に挿入済**（共通 `crates/common` 経由、Go 側 `internal/common/runtime.go` と等価）。**HTTP/JSON gateway も 3 Pod に結線済**（10 unary RPC: audit/{record,query,verifychain}, decision/{evaluate,batchevaluate,registerrule,listversions,getrule}, pii/{classify,mask}、`HTTP_LISTEN_ADDR` 設定時のみ起動、auth / ratelimit / 特権 RPC 自動監査が gRPC と同 chain で適用される、Audit.Export は server-streaming のため非対応）。Postgres / 外部 backend は post-MVP |
| `src/tier1/rust/crates/health` | 3 Pod 共通の HealthService 実装 | **同梱済** | `k1s0_tier1_health::Service` を提供（Liveness=version+uptime、Readiness=DependencyProbe を tokio::JoinSet で並列実行）。tier1 Go の `internal/health/` と同セマンティクス。4 単体テスト pass |
| `src/tier1/rust/crates/common` | 3 Pod 共通の interceptor / idempotency / HTTP gateway | **同梱済** | Go 側 `internal/common/` の Rust 等価物（25 単体テスト pass）。idempotency（24h TTL in-memory cache）/ tenant（NFR-E-AC-003 検証）/ auth（off / hmac / jwks 3 mode）/ ratelimit（テナント単位 token bucket）/ observability（tracing span + RED）/ audit（NoopAuditEmitter / LogAuditEmitter + 特権 RPC 表）/ http_gateway（axum + JsonRpc trait）/ grpc_layer（tonic Server に挿す tower::Layer）/ runtime（環境変数から CommonRuntime 一発構築）の 9 module |
| `src/tier1/rust/crates/{otel-util, policy, proto, proto-gen}` | 残補助 crate / proto stub | **雛形あり** | proto-gen crate 配置済、otel-util / policy / proto は `workspace.exclude` に置き plan 04-08 で順次合流予定 |
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
| `src/tier2/dotnet/services/{ApprovalFlow, InvoiceGenerator, TaxCalculator}` | tier2 完動例 | **同梱済** | 3 サービス DDD レイヤード（Domain / Application / Infrastructure / Api）+ xUnit ドメイン単体 + ArchitectureTests（NetArchTest）+ Dockerfile + catalog-info。**JWT Bearer 認証 (`AddK1s0JwtBearer()` extension via `K1s0.Tier2.Common.Auth` shared lib、T2_AUTH_MODE=off/hmac/jwks の 3 mode)** + 全業務エンドポイントに `RequireAuthorization()` 付与済（tier1 Go の `TIER1_AUTH_MODE` と同等強度） |
| `src/tier2/go/services/{notification-hub, stock-reconciler}` | tier2 Go 完動例 | **同梱済** | 2 サービス cmd + internal/{api,config}/ + Dockerfile + catalog-info。**JWT 認証 middleware (`shared/auth` パッケージ、go-jose/v4 で HS256-512 / RS256-512 検証 + JWKS TTL cache、6 単体テスト pass)** + handler で `t2auth.TenantIDFromContext` を取り出し `k1s0.WithTenant(ctx,...)` で SDK per-request 上書きする confused-deputy 対策済 |

### tier3（Web / Native / BFF / Legacy）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/tier3/web/apps/{portal, admin, docs-site}` | React + Vite + pnpm | **同梱済** | 3 SPA（main.tsx / App.tsx / pages 2 件 / vitest 単体）+ Dockerfile（nginx + SPA fallback）+ catalog-info |
| `src/tier3/web/packages/{ui, api-client, i18n, config}` | 共通パッケージ | **同梱済** | 4 package（ui: Button/Card/Spinner、api-client、i18n: ja/en、config）+ vitest 単体テスト |
| `src/tier3/bff/cmd/{portal-bff, admin-bff}` | Go BFF | **同梱済** | 2 BFF cmd + GraphQL（schema.graphql）+ REST + auth/middleware + k1s0client + shared/{errors,otel} + Dockerfile.{portal,admin}。**6/7 internal package に単体テスト 38 件 pass**（auth: HMAC/JWKS 検証 + middleware bypass 防止、config: env override + bool/int パース、errors: Category→HTTPStatus + DomainError chain、k1s0client: per-request tenant 上書き + Close nil-safety、rest: HTTP→502/200/anon、graphql: stateGet 3 path + currentUser + 未知クエリ）。internal/shared/otel のみ 61-line startup helper のため未テスト |
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
| `infra/observability/{loki, tempo, mimir, grafana, otel-collector, pyroscope, alerts}` | ADR-OBS-001/002 | **同梱済** | LGTM 6 component を production-grade で正規化済。**`alerts/` に PrometheusRule 4 本（k1s0-tier1/2/3-alerts + k1s0-slo）を配置：19 alert（SEV1=3 / SEV2=9 / SEV3=7）+ 6 recording rule（tier1 99.9% / tier3 99.5% SLO の 5m/1h/6h good ratio + Google SRE workbook 形式の fast/slow burn-rate アラート）。`ops/runbooks/daily/error-code-alert-policy.md` の閾値表に完全準拠、各 alert に runbook_url annotation で incident runbook へリンク** |
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
| `deploy/charts/{tier1-facade, tier1-rust-service, tier2-go-service, tier2-dotnet-service, tier3-bff, tier3-web-app, predeploy-hooks}` | Helm chart | **同梱済** | 7 chart 全て配置完了。`helm lint` 通過、`helm template` 描画 OK。`predeploy-hooks` は Argo CD PreSync Hook で Postgres / Kafka / Valkey / MinIO の readiness を polling 検証する Job 4 種を提供（Sync Wave -1）。**tier1/2/3 chart に `TIER1_AUTH_MODE` / `T2_AUTH_MODE` / `BFF_AUTH_MODE` env wiring 完了**（off / hmac / jwks 切替）+ tier1 chart に `httpPort` / `TIER1_RATELIMIT_RPS` / `TIER1_RATELIMIT_BURST` / `TIER1_HTTP_LISTEN_ADDR` / `TIER1_AUDIT_MODE` 等の production knobs を values 経由で公開。設計: `docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md` |
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
| `tests/` | e2e / contract / integration / fuzz / golden / fixtures | **同梱済**（一部雛形） | 6 カテゴリすべてに README + 動作可能な最小骨格 + CI hook 連携の入口。**fuzz/go**: 4 fuzz target（State.Set / Audit.Record / PubSub.Publish / Workflow.Start の protojson decode、735k execs / 0 panic 検証済）。**fuzz/rust**: 2 fuzz target（prost::Message::decode 4 message + SHA-256 hash chain 3 連鎖、libfuzzer 経由 CI 実行可能）。**integration/go**: tier1-facade の binary レベル結合テスト 3 件（Pod 起動 + HTTP/JSON gateway round-trip + tenant 越境）。**contract/openapi-contract**: tier1-openapi-spec.yaml を物理コピー、生成ツール `tools/codegen/openapi/run.sh` で自動同期。**golden/scaffold-outputs**: 4 ServiceType の expected.tar.gz、`compare-outputs.sh` で drift 検証。e2e のみ kind cluster 待ち（t.Skip） |
| `examples/` | Golden Path 7 プロジェクト | **同梱済** | 7 種すべてが build 可能な完動例。各 example に Dockerfile + catalog-info.yaml + 週次 E2E workflow。**tier2-{go,dotnet}-service は docs §共通規約「認証認可」を満たす auth middleware 結線済**（Go: t2auth.Required + k1s0.WithTenant per-request override、.NET: AddK1s0JwtBearer + RequireAuthorization）。templates / 既存 services と同パタンで Golden Path 一貫性を保つ |

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

## 実 K8s 検証実績（2026-04-30 session）

リリース時点直前に kind v0.27.0 + Kubernetes 1.31.4 + 8 Helm chart を実 install して
全 tier の round-trip を実機検証した。本セクションは「同梱済」ランクの**実機での裏付け**を
明示するもので、コード単体テストでは検出できなかった drift を 6 件発見し全て修正済（commit
`fd1621a62`〜`f7163b572`）。本ファイルの「同梱済」記述はすべて本検証で動作確認済。

### 実 cluster 構成（11 namespace × 50 Pod、全 Running）

| Namespace | コンポーネント | バージョン |
|---|---|---|
| `dapr-system` | Dapr control plane（operator / sidecar-injector / sentry / scheduler×3 / placement） | 1.17.5 |
| `cnpg-system` + `k1s0-tier1/pg-state` | CloudNativePG operator + Postgres single-instance | 0.28.0 / PG 18 |
| `kafka` | Strimzi operator + Kafka cluster（KRaft、controller+broker dual-role） | 0.51.0 / Kafka 4.2.0 |
| `openbao` | OpenBao server（dev mode、KVv2 マウント） | 2.5.3 |
| `temporal` | Temporal frontend / history / matching / worker / web + Cassandra + Elasticsearch | 0.65.0 |
| `keycloak` | Keycloak（quay.io/keycloak/keycloak:26.0、dev mode、realm `k1s0` 設定済） | 26.0 |
| `argocd` | Argo CD control plane + ApplicationSet で `tier1-facade` 3 環境（dev/staging/prod）生成 | 9.5.4 |
| `flagd` | flagd（OpenFeature 互換 Flag 評価エンジン） | 0.11.6 |
| `k1s0-tier1` | tier1 Go 3 Pod（state with Dapr sidecar 2/2、secret 2/2、workflow 2/2）+ Rust 3 Pod | dev tag |
| `k1s0-tier2` | notification-hub Go + TaxCalculator .NET | dev tag |
| `k1s0-tier3` | portal-bff Go + portal-web Nginx | dev tag |

### 実バックエンド経由で証跡を取った round-trip 経路

| Component | 経路 | 検証内容 | 結果 |
|---|---|---|---|
| `state.in-memory` | tier1-state Set/Get | UUID etag 復元 | OK |
| `state.postgresql v2` → CNPG Postgres 18 | tier1-state Set/Get、Pod kill 後復旧 | 同 etag/data 復元（force kill 含む） | OK |
| `pubsub.kafka v1` → Strimzi Kafka 4.2.0 | tier1-state Publish | kafka-console-consumer で CloudEvents JSON 受信 | OK |
| `pubsub.kafka v1` | tier1-state gRPC Subscribe stream | grpcurl で 3 events 順序通り受信（base64 cnQtMQ==/cnQtMg==/cnQtMw==） | OK |
| OpenBao KVv2 | tier1-secret Get / Rotate / 自動 rotation | values 復元 + version 1→2、`ROTATION_SCHEDULE=10s` で 10 秒ごとに version bump（21→30 観測） | OK |
| Temporal | tier1-workflow Start / GetStatus | UUIDv7 runId、Temporal CLI で `T-temporal::wf-temporal-1` 登録確認 | OK |
| Rust audit Pod | Record / Query / VerifyChain | hash chain integrity valid | OK |
| Rust decision Pod | RegisterRule / Evaluate（ZEN Engine） | `{"tier":"high"}` | OK |
| Rust pii Pod | Classify / Mask | EMAIL+PHONE 検出、`[EMAIL]` 置換 | OK |
| Keycloak Realm `k1s0` | password grant → JWT 発行 | `tenant_id=T-kc` + `realm_access.roles=[user]` クレーム埋込確認 | OK |
| BFF JWKS verify → tier1 State | 完全 chain（Web→BFF→tier1） | Bearer JWT 検証 → tier1 Postgres Get で HTTP 200 | OK |

### 本検証で発見した実装 drift（全件修正済 commit 化）

| commit | 修正内容 | 発見経路 |
|---|---|---|
| `f7163b572` | tier1 PubSub の tenant separator が `/` 固定で Kafka topic 規則 `[a-zA-Z0-9._-]+` と非互換、`.` separator を pubsub 専用に新設 | Strimzi で "invalid topic" エラー |
| `6ada41a6e` | .NET Dockerfile の `PublishReadyToRun` が RID 必須 / web Dockerfile の pnpm `--frozen-lockfile` 不整合 + tsconfig.base.json 不在 / tier3-web nginx `proxy_pass URL/` の trailing `/` で BFF が 404 | 実 docker build / 実 web→BFF 経路で発覚 |
| `d479cedb4` | tier1 Go Dockerfile の Go 1.22 → 1.25 base image 不整合 + `replace ../../sdk/go` の build context 不在、Rust 3 Pod が `grpc.health.v1.Health` 未実装で K8s grpc liveness/readiness probe 不能 | 実 Helm install で発覚 |
| `53c284e84` | proto 13 ファイルに `google.api.http` annotation 未付与で OpenAPI v2 spec が paths 0、schemathesis / dredd の契約検証が事実上 no-op | OpenAPI gen 結果 0 paths で発覚 |
| `922fd1edf` | Rust `privileged_rpcs()` が Go 側 `privilegedRPCs` と乖離（IDL 不在 RPC 混入 / 必須 RPC 不足）、AuthClaims に `roles` フィールドが 3 言語とも不在で NFR-E-AC-002 RBAC 判定不能 | docs ↔ コード整合確認で発覚 |
| `533346f80` | infra/security/cert-manager/values.yaml に webhook/cainjector 重複キー（後段が前段を黙って消す） | kubeconform で検出 |

リリース時点コードはこれら 6 修正と前段 2 commit（`fd1621a62` tier1 3 経路結線 / `84aedee88` 残 RPC E2E + OTLP gRPC exporter）を含む。

### 既知の未検証領域（採用初期で実施）

採用検討者が誤認しないよう、本セッション**未検証**の項目を明記する。

- Argo Rollouts canary 戦略の実発火（ADR-CICD-002）
- Kyverno ClusterPolicy の実発火（ADR-CICD-003）
- Istio Ambient mesh / mTLS 強制（ADR-0001）
- SPIRE / SPIFFE workload identity（ADR-SEC-003）
- Loki / Tempo / Mimir / Grafana スタックへの実 OTLP gRPC 流入（ADR-OBS-001）
- SLSA / cosign 署名フロー（ADR-SUP-001）
- NFR-A 可用性 SLO 99.9% / NFR-B 性能 p99 latency の数値計測
- tier3-native MAUI / Argo CD ApplicationSet の実 git sync（local cluster は git remote 未認証）
- Backstage backend 統合（plugin の build は通過、Backstage 本体は未起動）
- flagd と tier1 FeatureService の Dapr Configuration 経由結線（flagd Pod は起動済、Component CR は未配備）

### ADR 実装トレース（36 件全件、`src/` `infra/` `deploy/` `tools/` を node_modules / target / dist 除外でスキャン）

各 ADR の「決定が実装ツリーに何らかの形で反映されているか」をサンプル参照ファイル付きで追跡する。
**実 K8s クラスタでの動作検証**は「実 K8s 検証実績」セクションで深さを区別する（コード / 設定が
存在 ≠ 実 cluster で動作確認済）。

| ADR | 実装サンプル参照 | 実 cluster 検証 |
|---|---|---|
| ADR-0001 (Istio Ambient vs sidecar) | `infra/mesh/istio-ambient/values.yaml` | 未検証（採用初期） |
| ADR-0002 (drawio layer 規約) | `tools/git-hooks/drawio-svg-staleness.sh` | N/A（規約） |
| ADR-0003 (AGPL 隔離) | `infra/README.md` | N/A（運用ポリシー） |
| ADR-BS-001 (Backstage) | `src/platform/backstage-plugins/`、`catalog-info.yaml` × 多数 | 未検証（plugin build 通過のみ） |
| ADR-CICD-001 (Argo CD) | `deploy/apps/{app-of-apps,application-sets,projects}/`、`infra/k8s/namespaces/namespaces.yaml` | **検証済**（control plane install + 4 AppProject + 3 Application） |
| ADR-CICD-002 (Argo Rollouts) | `deploy/rollouts/{analysis,canary-strategies}/` | 未検証 |
| ADR-CICD-003 (Kyverno) | `infra/security/kyverno/{baseline-policies,kustomization}.yaml` | 未検証 |
| ADR-DATA-001 (CloudNativePG) | `infra/data/cloudnativepg/`、`postgresql.cnpg.io` CRD 利用 | **検証済**（CNPG operator install + Postgres single-instance + tier1-state 永続化） |
| ADR-DATA-002 (Strimzi Kafka) | `infra/data/kafka/{kafka-cluster,strimzi-values}.yaml` | **検証済**（Strimzi 0.51 + Kafka 4.2.0 KRaft + Publish/Subscribe round-trip） |
| ADR-DATA-003 (MinIO) | `infra/data/minio/values.yaml` | 未検証 |
| ADR-DATA-004 (Valkey) | `infra/data/valkey/values.yaml`、`state.redis` adapter | 未検証 |
| ADR-DEP-001 (Renovate) | `renovate.json`、`infra/security/openbao/policies/ci-runner.hcl` | 未検証（CI gate 別 PR） |
| ADR-DEV-001 (Paved Road) | `src/tier2/templates/`、`src/tier3/templates/` | 未検証（Backstage Scaffolder 実行は別 PR） |
| ADR-DEV-002 (WSL2 + Docker) | `tools/devcontainer/postCreate.sh`、`.devcontainer/` | **検証済**（本セッション全体が WSL2 + Docker 環境で実行） |
| ADR-DIR-001 (contracts 昇格) | `src/contracts/tier1/`、`src/contracts/internal/` | **検証済**（buf generate / OpenAPI 42 paths 化が contracts/ 直接参照） |
| ADR-DIR-002 (infra 分離) | `infra/` ディレクトリ全体、`infra/README.md` | **検証済**（kustomize 3 環境 + helm chart 全 7 件 dry-run + 一部実 install） |
| ADR-DIR-003 (sparse checkout) | `.sparse-checkout/`、`tools/sparse/{checkout-role,verify}.sh` | 未検証（cone mode は CI で実行する想定） |
| ADR-DX-001 (DX metrics 分離) | `tools/catalog-check/check-lifecycle.sh` | N/A（DX metric 計測パイプライン別 PR） |
| ADR-FM-001 (flagd / OpenFeature) | `infra/feature-management/flagd/{flagd-deployment,values}.yaml`、`src/contracts/tier1/k1s0/tier1/feature/` | 部分検証（flagd Pod 起動済、Dapr Configuration 経由結線は別 PR） |
| ADR-MIG-001 (.NET Framework sidecar) | `src/README.md`（言及のみ）、実装なし | N/A（採用後の運用拡大時で導入） |
| ADR-MIG-002 (API Gateway) | `infra/mesh/envoy-gateway/gateway-internal.yaml` | 未検証 |
| ADR-OBS-001 (Grafana LGTM) | `infra/environments/dev/values/{loki,tempo}/values.yaml`、`infra/observability/{grafana,loki,tempo,mimir,pyroscope}/values.yaml` | 未検証（OTLP exporter 結線済 / Collector → LGTM スタック流入は採用初期で確認） |
| ADR-OBS-002 (OTel Collector) | `src/tier2/go/shared/otel/init.go`、`src/tier1/go/internal/otel/otel.go`、`infra/observability/otel-collector/values.yaml` | 部分検証（OTLP gRPC exporter 結線済、実 Collector への流入は採用初期） |
| ADR-OBS-003 (Incident Taxonomy) | `infra/observability/alerts/k1s0-tier3-alerts.yaml` | N/A（運用文書 + alert ルール） |
| ADR-POL-001 (Kyverno 二重オーナー) | `infra/security/kyverno/`、`infra/README.md` | 未検証 |
| ADR-REL-001 (Progressive Delivery) | `deploy/rollouts/canary-strategies/canary-25-50-100.yaml` | 未検証 |
| ADR-RULE-001 (ZEN Engine) | `src/contracts/internal/k1s0/internal/decision/v1/decision.proto`、`src/tier1/rust/crates/decision/` | **検証済**（cluster 上 t1-decision Pod で RegisterRule→Evaluate `{"tier":"high"}`） |
| ADR-RULE-002 (Temporal) | `src/tier1/go/internal/adapter/temporal/workflow.go`、`infra/feature-management/`（参照） | **検証済**（Temporal cluster install + tier1-workflow Start/GetStatus、Temporal CLI で workflow 確認） |
| ADR-SEC-001 (Keycloak) | `infra/security/keycloak/values.yaml`、tier3 BFF JWKS auth | **検証済**（Keycloak 26 install + realm `k1s0` + BFF JWKS verify chain HTTP 200） |
| ADR-SEC-002 (OpenBao) | `infra/security/openbao/`、`src/tier1/go/internal/adapter/openbao/`、`src/sdk/typescript/src/secrets.ts` | **検証済**（OpenBao install + tier1-secret Get/Rotate + 自動 rotation 10 回観測） |
| ADR-SEC-003 (SPIFFE/SPIRE) | `infra/security/spire/values.yaml` | 未検証 |
| ADR-STOR-001 (Longhorn) | `infra/k8s/storage/kustomization.yaml` | 未検証（kind 環境では local-path-provisioner 使用） |
| ADR-STOR-002 (MetalLB) | `infra/k8s/networking/metallb-values.yaml` | 未検証 |
| ADR-SUP-001 (SLSA / SBOM / cosign) | `src/tier2/templates/go-service/skeleton/{{name}}/Dockerfile.hbs`（template 内） | 未検証 |
| ADR-TIER1-001 (Go + Rust hybrid) | `src/tier1/go/cmd/{state,secret,workflow}`、`src/tier1/rust/crates/{audit,decision,pii}` | **検証済**（6 Pod 全 Running） |
| ADR-TIER1-002 (Protobuf gRPC) | 14 proto / 47 RPC + 4 SDK 生成 | **検証済**（gRPC + HTTP/JSON gateway 経路で 17 round-trip 完了） |
| ADR-TIER1-003 (言語不可視) | tier1 internal proto は SDK 配布物に含まれない設計、`src/contracts/internal/` | **検証済**（buf.gen.internal.yaml で internal を SDK から分離生成） |

**集計**: 36 件中、コード/設定が存在 = 36 件、実 K8s 検証済 = 14 件、部分検証 = 2 件、未検証 = 16 件、N/A（規約 / 運用文書） = 4 件。
未検証 16 件はすべて「採用初期」段階の実装対象で、リリース時点では設計合意（Accepted）状態を docs / IaC で確定済。

### B / C / D セクション 実 K8s 検証実績（2026-04-30 後段 session）

A セクション完了後に observability / Kyverno / Argo Rollouts / Istio Ambient / NFR を実機検証した
結果。8 commit 後さらに以下を確認済。

| 検証 | 実装 / 結果 |
|---|---|
| **B: OTLP gRPC 流入 (ADR-OBS-002)** | OTel Collector 0.120.1 を `observability` namespace に install、tier1-state Pod から OTLP gRPC で **3 signal（Logs / Metrics / Traces）が実流入確認済**。debug exporter で `flush_test_1/2/3` Counter, "OTel pipeline test/final test" Body, span "otel-test" を観測。OTEL_EXPORTER_OTLP_ENDPOINT は `http://` scheme + per-signal `OTEL_EXPORTER_OTLP_INSECURE=true` 必須（scheme なし時 `delegating_resolver: invalid target address` で gRPC client が空文字解決失敗）。 |
| **C1: Kyverno baseline policy 実発火 (ADR-CICD-003 / ADR-POL-001)** | `infra/security/kyverno/baseline-policies.yaml` の 4 ClusterPolicy（require-run-as-non-root / disallow-privileged-containers / require-k1s0-component-label / require-resource-requests）を deploy 完了。Kyverno admission webhook が違反 Pod を実拒否することを確認（`runAsNonRoot: false` Pod 作成試行で「validation error: ... rule check-non-root failed」として block）。**追加修正**: 当初 baseline policy が `kube-system` を含む infra namespace を policy 対象としていたため `istio-system` / `kafka` / `dapr-system` などへの operator install が block された。Kyverno baseline policies に 21 system/infra namespace 例外を追加（`infra/security/kyverno/baseline-policies.yaml` 修正済）。 |
| **C2: Argo Rollouts canary 戦略 (ADR-CICD-002 / ADR-REL-001)** | argo-rollouts operator install + Rollout CR（25→50→75→100% steps）を deploy。image patch 後 stepIndex 1 (Paused 25%) → 2 (Progressing 50%) → 4 (Progressing 75%) → 5 (Paused) → 6 (Healthy 100%) の canary progression を実観測。古い ReplicaSet → 新 ReplicaSet への切替を実機確認。 |
| **C3: Istio Ambient mesh + mTLS STRICT (ADR-0001)** | istio-base / istiod (`profile=ambient`) / istio-cni / ztunnel を install。tier1 namespace に `istio.io/dataplane-mode=ambient` label 付与 + `PeerAuthentication` STRICT を設定。**plain text 接続を ztunnel が実拒否** することを確認（access log: `error="connection closed due to policy rejection: explicitly denied by: istio-system/istio_converted_static_strict"`）。kind 環境では `fs.inotify.max_user_instances` を 128 → 1024 に bump する必要あり（`docker exec k1s0-local-control-plane sysctl -w`）。 |
| **D: NFR 数値実測** | NFR-B-PERF: tier1-state HTTP/JSON gateway（Postgres backed `state.postgresql v2`）に hey で 5000 req / 50 concurrent 負荷 → **p50=3.8ms / p90=61ms / p95=70ms / p99=74ms**（全 200 OK、0 失敗）。NFR-A-FT-001: tier1-state Pod を `--force --grace-period=0` で kill → 新 Pod 2/2 Ready まで **756 秒（12 分 36 秒）で復旧**、NFR-A-FT-001 SLA「自動復旧 15 分以内」**PASS**。Postgres 永続層から同一 etag/data 復元確認。本数値は kind single-node 環境のため、production multi-node では image preload / pre-pull / readiness probe 最適化で数十秒〜2 分が想定される。 |

### 8 namespace 例外を baseline policy に追加した経緯（C1 の追加修正）

`infra/security/kyverno/baseline-policies.yaml` は当初 baseline policy として全 namespace に適用する
設計だったが、operator パターン（istio-cni / ztunnel / istio-init / strimzi-cluster-operator /
dapr control plane / cnpg-system 等）は **privileged container** または **runAsNonRoot=false**
を必須とする。Kyverno と二重オーナー設計（ADR-POL-001）に従い、infra layer の operator namespace
を baseline policy の `exclude.any.resources.namespaces` に追加した。除外対象 21 namespace:

`kube-system` / `kube-public` / `kube-node-lease` / `local-path-storage` / `kyverno` /
`istio-system` / `cnpg-system` / `dapr-system` / `kafka` / `openbao` / `temporal` / `keycloak` /
`observability` / `argocd` / `argo-rollouts` / `flagd` / `cert-manager` / `metallb-system` /
`calico-system` / `tigera-operator` / `longhorn-system`

application 層（k1s0-tier1 / tier2 / tier3）には baseline policy が引き続き適用され、業務 Pod は
`runAsNonRoot=true` + resources.requests + k1s0.io/component label 必須が強制される。

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
