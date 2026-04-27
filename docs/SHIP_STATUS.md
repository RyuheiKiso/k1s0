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
| `src/contracts/tier1/` proto 12 サービス | 12 API（state / pubsub / serviceinvoke / secrets / binding / workflow / log / telemetry / decision / audit / feature / pii） | **同梱済** | 12 サービスの正式 RPC を `docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/` 正典から移植済（合計 43 RPC、`buf lint` / `buf format` 通過、4 SDK 再生成済）。共通型は `common/v1/common.proto`（TenantContext / ErrorDetail / K1s0ErrorCategory）に集約。Audit と PII は IDL 上 1 ファイルだがディレクトリ設計に従い 2 パッケージに分割 |
| `src/contracts/internal/` proto | tier1 内部 gRPC（Go ↔ Rust core） | **雛形あり** | placeholder 1 ファイル |
| `src/tier1/go/cmd/{state,secret,workflow}/` | Go 側 3 Pod（DS-SW-COMP-005/006/010、6 Pod 構成のうち Go 担当分） | **雛形あり** | gRPC server bootstrap + standard health protocol + reflection + graceful shutdown 動作。**全 7 公開 API の handler 登録完了**（t1-state: 5 / t1-secret: 1 / t1-workflow: 1）。各 RPC は `internal/adapter/dapr/` 経由で `codes.Unimplemented` を返す（Dapr backend 結線は plan 04-04〜04-13 で順次）。Log / Telemetry は t1-state Pod の内部 adapter（DS-SW-COMP-037/038）として収容される設計 |
| `src/tier1/go/internal/common/` | 共通 runtime（gRPC bootstrap / config / retry / timeout） | **同梱済** | runtime / config / retry / timeout の 4 ユーティリティとテストが存在 |
| `src/tier1/go/internal/otel/` | OTel 初期化 | **雛形あり** | 1 ファイルの最小骨格 |
| `src/tier1/rust/crates/{decision, audit, pii}` | Rust 側 3 Pod（DS-SW-COMP-008/007/009、6 Pod 構成のうち Rust 担当分） | **雛形あり** | 3 crate（k1s0-tier1-decision / k1s0-tier1-audit / k1s0-tier1-pii）の Cargo.toml + src/main.rs を配置。tonic server を :50001 で起動し公開 Service trait を登録、graceful shutdown 対応。**全 RPC は Status::unimplemented を返す**（実 ZEN Engine / Postgres WORM / PII 検出ロジックは plan 04-08〜04-10 で順次） |
| `src/tier1/rust/crates/{common, otel-util, policy, proto, proto-gen}` | 共通 crate / proto stub | **雛形あり** | proto-gen crate（k1s0-tier1-proto-gen）を配置し buf 生成 internal proto を Rust module 階層に束ねた。common / otel-util / policy / proto は plan 04-02 / 04-08 等で内容実装に追従し workspace.members に追加予定（現在は exclude） |
| Dockerfile（distroless / nonroot / multi-stage） | 3 Pod 各 1 Dockerfile | **同梱済** | `Dockerfile.{state,secret,workflow}` は完成 |

### contracts と SDK 生成

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `buf.gen.yaml` | 4 言語生成設定（Go / .NET / Rust / TypeScript） | **同梱済** | tier1 公開と internal の 2 module 構成 |
| `src/sdk/go/generated/` | Go gRPC stub 生成 | **同梱済** | 12 サービス分の正式 RPC 群を生成済（28 ファイル: pb.go + grpc.pb.go の 14 proto 分） |
| `src/sdk/dotnet/generated/` | .NET stub | **同梱済** | 12 サービス分の正式 RPC 群を生成済（28 ファイル） |
| `src/sdk/rust/generated/` | Rust prost / tonic stub | **同梱済** | 12 サービス分の正式 RPC 群を生成済（28 ファイル: prost + tonic の 14 proto 分） |
| `src/sdk/typescript/generated/` | TS protobuf-es / connect-es stub | **同梱済** | 12 サービス分の正式 RPC 群を生成済（28 ファイル: pb.ts + connect.ts の 14 proto 分） |
| 高水準 SDK（`k1s0.State.Save(...)` 等の動詞統一） | docs 規定の 4 言語動詞 | **同梱済**（14 service 全件） | 4 言語すべてに Client + 動詞統一 facade を 14 service 全件（公開 12 + Admin 2）で実装。公開: State / PubSub / Secrets / Log / Workflow / Decision / Audit / Pii / Feature / Binding / ServiceInvoke / Telemetry。Admin: DecisionAdmin（RegisterRule / ListVersions / GetRule）/ FeatureAdmin（RegisterFlag / GetFlag / ListFlags）。Go `c.DecisionAdmin().RegisterRule()` / `c.FeatureAdmin().GetFlag()`、Rust `client.decision_admin().register_rule()`、TypeScript `client.decisionAdmin.registerRule()`、.NET `client.DecisionAdmin.RegisterRuleAsync()`。Go `go build ./...` / Rust `cargo verify-project` / TypeScript `pnpm build` / .NET `dotnet build Sdk.sln` 全通過。Stream RPC（InvokeStream / PubSub.Subscribe）は callback / Streaming / AsyncIterable / IAsyncEnumerable で同梱 |

### tier2（C# / Go ドメイン共通）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/tier2/templates/{dotnet,go}-service` | scaffold が参照する型付きテンプレート | **設計のみ** | ディレクトリ存在のみ |
| `src/tier2/dotnet/services/{ApprovalFlow, InvoiceGenerator, TaxCalculator}` | tier2 完動例 | **設計のみ** | ディレクトリ存在のみ |
| `src/tier2/go/services/{notification-hub, stock-reconciler}` | tier2 Go 完動例 | **設計のみ** | ディレクトリ存在のみ |

### tier3（Web / Native / BFF / Legacy）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/tier3/web/apps/{portal, admin, docs-site}` | React + Vite + pnpm | **設計のみ** | ディレクトリ存在のみ |
| `src/tier3/web/packages/{ui, api-client, i18n, config}` | 共通パッケージ | **設計のみ** | ディレクトリ存在のみ |
| `src/tier3/bff/cmd/{portal-bff, admin-bff}` | Go BFF | **設計のみ** | ディレクトリ存在のみ |
| `src/tier3/native/apps/{Hub, Admin}` | .NET MAUI | **設計のみ** | ディレクトリ存在のみ |
| `src/tier3/legacy-wrap/sidecars/K1s0.Legacy.Sidecar` | .NET Framework サイドカー | **設計のみ** | ディレクトリ存在のみ |

### platform（CLI / Backstage プラグイン / Analyzer）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `src/platform/cli/` | k1s0-scaffold 雛形 CLI | **設計のみ** | ディレクトリ存在のみ |
| `src/platform/analyzer/` | 内製依存方向 analyzer（`tier3 → tier2 → tier1 → infra` 一方向強制） | **設計のみ** | ディレクトリ存在のみ |
| `src/platform/backstage-plugins/` | Backstage 開発者ポータル plugin | **設計のみ** | ディレクトリ存在のみ |

### infra（k8s / mesh / data / observability / security）

| 領域 | docs 規定（リリース必須） | 実装ランク | 備考 |
|---|---|---|---|
| `infra/k8s/{bootstrap, namespaces, networking, storage}` | kubeadm HA + Calico/MetalLB | **雛形あり** | bootstrap: Cluster API（CAPI）+ KubeadmControlPlane（HA 3 + audit log + OIDC issuer）と kubeadm-init 代替。namespaces: 17 layer 全 namespace に Pod Security Standards label（restricted/baseline/privileged）+ istio Ambient ラベル。networking: Calico VXLAN（Pod CIDR 192.168.0.0/16）+ MetalLB（IPAddressPool は環境別 overlay）。storage: 4 種 StorageClass（default / high-iops / shared / backup） |
| `infra/mesh/{istio-ambient, envoy-gateway}` | ADR-0001 / ADR-MIG-002 | **雛形あり**（istio-ambient のみ） | istio-ambient: profile ambient + istiod HA 3 replica + HPA + OTel tracing 連携 + ztunnel 設定 + mTLS STRICT 強制 PeerAuthentication。envoy-gateway は本リリース時点 未着手 |
| `infra/dapr/control-plane/` | Dapr operator | **雛形あり** | HA 全 control-plane component（operator/placement/sentry/sidecar-injector/scheduler）3 replica + mTLS + Prometheus + Raft consensus（placement）|
| `infra/data/{cloudnativepg, kafka, minio, valkey}` | ADR-DATA-001/002/003/004 | **雛形あり** | 4 backend を production-grade defaults（HA / 監視 / バックアップ）で正規化済。CNPG: 3 instance HA + WAL アーカイブ + PodMonitor。Kafka: Strimzi KRaft 3 broker + TLS mTLS + Cruise Control。MinIO: distributed mode 4 replica + erasure coding + ServiceMonitor。Valkey: replication + Sentinel + 認証有効。各ディレクトリに `values.yaml` + `<backend>-cluster.yaml` + `README.md`（dev 差分表 / ADR リンク）を配置 |
| `infra/security/{cert-manager, keycloak, openbao, spire, kyverno}` | ADR-SEC-001/002/003 / ADR-POL-001 | **雛形あり** | cert-manager: HA 3 + Let's Encrypt prod/staging + 内部 CA ClusterIssuer。Keycloak: HA 3 + Infinispan + 外部 CNPG + production mode + ServiceMonitor。SPIRE: server HA 3 + 外部 CNPG + cert-manager upstream + CSI driver + OIDC Discovery。Kyverno: admission HA 3 + 4 baseline ClusterPolicy。OpenBao: HA 3 + Raft integrated storage + cert-manager TLS + Vault Agent Injector + UI、HCL policy 4 種既存 |
| `infra/observability/{loki, tempo, mimir, grafana, otel-collector, pyroscope}` | ADR-OBS-001/002 | **雛形あり** | LGTM 6 component を production-grade defaults で正規化済。Loki SimpleScalable + S3、Tempo distributed + S3、Mimir distributed + S3、Pyroscope micro-services + S3、Grafana HA + Keycloak OIDC、OTel Collector agent DaemonSet + gateway Deployment 2 段（tail_sampling + 4 backend ファンアウト）。各ディレクトリに `values.yaml` + 説明、`infra/observability/README.md` に信号フロー図と dev 差分表 |
| `infra/scaling/keda/` | KEDA | **雛形あり** | operator HA 2 + metrics-apiserver 2 + admission webhook + ServiceMonitor、Kafka topic lag / Postgres queue / per-tenant RPS でオートスケール想定 |
| `infra/feature-management/flagd/` | ADR-FM-001 | **雛形あり** | flagd HA 3 replica Deployment + ConfigMap baseline flag + Service + ServiceMonitor。tier1 t1-state Pod の Feature handler が gRPC で参照 |
| `infra/environments/{dev, staging, prod}` | 環境別 overlay | **設計のみ** | `.gitkeep` のみ |

### deploy（GitOps / Helm / Kustomize / OpenTofu）

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `deploy/apps/{application-sets, projects}` | Argo CD ApplicationSet（リリース必須、ADR-CICD-001） | **雛形あり** | tier1-facade 用 ApplicationSet を最小同梱 |
| `deploy/charts/{tier1-facade, tier1-rust-service, tier2-go-service, tier2-dotnet-service, tier3-bff, tier3-web-app}` | Helm chart（リリース必須） | **雛形あり** | 6 chart 全て配置完了。tier1-facade（既存、Go 3 Pod）/ tier1-rust-service（Rust 3 Pod ループ）/ tier2-go-service（汎用 Go テンプレート）/ tier2-dotnet-service（汎用 .NET テンプレート、aspnet runtime + ASPNETCORE_URLS）/ tier3-bff（Go BFF + OIDC + ingress）/ tier3-web-app（nginx + SPA fallback + BFF reverse proxy）。`helm lint` 全 5 chart 通過、`helm template` 描画 OK |
| `deploy/rollouts/{canary-strategies, analysis-templates, experiments}` | Argo Rollouts（リリース必須、ADR-CICD-002） | **雛形あり** | canary 25→50→100% の 3 段階戦略テンプレート + AnalysisTemplate 2 件（error-rate / latency-p99、Mimir Prometheus クエリ）。experiments は採用後の運用拡大時 で追加 |
| `deploy/kustomize/{base, overlays/*}` | Kustomize | **雛形あり** | base（共通 Namespace + label）+ overlays/{dev,staging,prod}/ に 6 chart × 3 環境 = 18 values overlay と 3 kustomization.yaml + README を配置。dev: replica=1 / debug、staging: replica=2 / info / HPA、prod: replica=3 / warn / podAntiAffinity / image semver pinning |
| `deploy/opentofu/{environments, modules}` | OpenTofu（採用後の運用拡大時に Terraform から移行） | **設計のみ** | `.gitkeep` のみ |
| `deploy/image-updater/` | Argo CD Image Updater | **雛形あり** | argocd-image-updater Helm values（HA 2 + ServiceMonitor）+ registries.conf ConfigMap（GHCR + GHCR pull secret 連携）+ application-annotations.md（ApplicationSet への annotation 追加例、環境別 strategy 表、Renovate との責任分界）を配置 |

### tools / tests / examples

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `tools/local-stack/` | kind ベース本番再現スタック（IMP-DEV-POL-006） | **同梱済** | `up.sh` / `down.sh` / `status.sh` / `kind-cluster.yaml` / 17 レイヤ namespace yaml |
| `tools/local-stack/manifests/{20..95}_*/` | 各レイヤの Helm values / Kustomize | **同梱済** | 17 レイヤ全てに `values.yaml`（Helm 用）または `manifest.yaml`（Kustomize / 直 apply 用）配置済 |
| `tools/devcontainer/` | 10 役 Dev Container プロファイル | **雛形あり** | `postCreate.sh` / `doctor.sh` / README は存在 |
| `tools/sparse/` | sparse-checkout 10 役 cone 定義 | **雛形あり** | `checkout-role.sh` / `verify.sh` / README は存在 |
| `tools/codegen/` | buf / openapi / grpc-docs 生成ラッパ | **設計のみ** | Makefile が呼ぶ `tools/codegen/buf/run.sh` 等は未配置 |
| `tools/git-hooks/` | 自作 pre-commit hook | **同梱済** | `japanese-header-guard.py` / `file-length-guard.py` / `drawio-svg-staleness.sh` / `link-check-wrapper.py` |
| `tools/_link_check.py` / `_link_fix.py` / `_export_svg.py` | docs 横断ツール | **同梱済** | docs リンク検査・drawio SVG export |
| `tests/` | e2e / contract / integration / fuzz / golden | **設計のみ** | tier1 Go 内に unit test (config / retry / timeout) のみ |
| `examples/` | Golden Path 7 プロジェクト（tier1-rust-service / tier1-go-facade / tier2-{dotnet,go}-service / tier3-{web-portal, bff-graphql, native-maui}） | **雛形あり** | 7 種すべてが build 可能な完動例として配置。tier1-rust-service / tier1-go-facade / tier3-web-portal は最小実装（既存）、tier2-go-service（k1s0 SDK State.Save HTTP endpoint）/ tier2-dotnet-service（ASP.NET Core minimal API）/ tier3-bff-graphql（GraphQL 経由 State.Get）/ tier3-native-maui（MAUI ViewModel）を追加完成。各 example に Dockerfile + catalog-info.yaml + 週次 E2E workflow（`.github/workflows/example-<name>.yml` 7 種）を配置 |

### CI / CD / GitOps

| 領域 | docs 規定 | 実装ランク | 備考 |
|---|---|---|---|
| `.github/workflows/pr.yml` | path-filter で 11 軸検出 + reusable workflow 呼び出し + ci-overall 集約（IMP-CI-POL-002 / 003） | **同梱済** | path-filter / reusable 4 本 / commitlint まで構成済 |
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
2. **ID 体系**: 要件 ID（FR-T1-* / NFR-* / BR-* / etc.）と設計 ID（DS-* / IMP-*）は実装側コミットメッセージにも追跡される（IMP-TRACE-*）
3. **トレーサビリティ索引**: `docs/03_要件定義/80_トレーサビリティ/` と `docs/04_概要設計/80_トレーサビリティ/` で要件 → 設計 → ADR の対応が網羅される
4. **CI ゲート**: `buf lint` / `buf breaking` / 内製 analyzer / pre-commit が逸脱を物理的に遮断する

## 採用検討者へのガイダンス

- **「docs を信じて全部動く」前提では採用しない**こと。本ファイルの「同梱済」のみを動作前提として評価してほしい
- POC 用途では `tools/local-stack/up.sh --role docs-writer`（最小構成）または `--role tier1-rust-dev`（tier1 検証構成）から始めることを推奨する
- 業務適用には「採用初期」段階の実装完了が必要。tier1 全 Pod ハンドラ・Helm chart 全種・examples 完動 4 種が完成してから本番投入を検討すること
- 実装が進むに従い本ファイルは更新される。最新版は main ブランチの本ファイルを参照

## 次の段階で進めるべき作業（採用初期へのロードマップ）

リリース時点（v0.x）から採用初期段階へ進むために、以下を**docs を逸脱せず**実装する必要がある。
順序は依存方向（contracts → SDK → tier1 → tier2/tier3）と外部評価の費用対効果を考慮した推奨順。

### 1. contracts の正式 RPC 化（IMP-CODEGEN / FR-T1-*）— **完了**

- `src/contracts/tier1/*/v1/*.proto` の各サービスを `PlaceholderCall` 1 RPC から
  正式 RPC 群に置換済（IDL 正典 `docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/` 移植）。
  - 12 サービス計 43 RPC（state 5 / pubsub 3 / serviceinvoke 2 / secrets 3 /
    binding 1 / workflow 6 / log 2 / telemetry 2 / decision 5 / audit 2 /
    pii 2 / feature 7、別途 health 2 RPC）
  - 共通型 `common.proto` 集約（旧 `principal.proto` / `page.proto` を削除し
    `TenantContext` / `ErrorDetail` / `K1s0ErrorCategory` に統一）
  - IDL 命名規約と `buf` STANDARD lint の衝突箇所は `src/contracts/buf.yaml`
    の `lint.except` で除外（IDL 正典を優先する方針を明記）
- `make codegen`（`buf generate`）で 4 SDK を再生成済（各 28 ファイル × 4 言語）
- `buf lint` / `buf format` 通過確認済
- `buf breaking` は v0 placeholder → v1 正式 RPC への移行を意図した破壊的変更を検出。
  本コミットを v1 のベースラインとし、以降の PR は本コミット以降の差分に対して `buf breaking` を強制

### 2. SDK モジュール化（IMP-CODEGEN / IMP-DIR）— **完了**

- 生成先パスを docs 正典に揃えた（旧 `src/sdk/*/generated/` フラット配置 → docs 正典の言語別構造化パス）:
  - Go:  `src/sdk/go/proto/v1/k1s0/tier1/<api>/v1/`
  - Rust: `src/sdk/rust/crates/k1s0-sdk-proto/src/gen/v1/`
  - TS:   `src/sdk/typescript/src/proto/k1s0/tier1/<api>/v1/`
  - .NET: `src/sdk/dotnet/src/K1s0.Sdk.Proto/Generated/`
- 4 言語の module / workspace を配置:
  - **Go**: `src/sdk/go/go.mod`（module `github.com/k1s0/sdk-go`、Go 1.22、`go build ./...` 通過）
  - **Rust**: `src/sdk/rust/Cargo.toml` workspace（edition 2024）+ `crates/k1s0-sdk-proto/`（生成 stub）+ `crates/k1s0-sdk/`（薄い facade）、`cargo metadata` 通過
  - **TypeScript**: `src/sdk/typescript/package.json`（`@k1s0/sdk-rpc`、Node 20+）+ `tsconfig.json` + `tsconfig.build.json`、`tsc --noEmit` と build 両方通過
  - **.NET**: `src/sdk/dotnet/Sdk.sln` + `src/K1s0.Sdk.Proto/`（生成 stub）+ `src/K1s0.Sdk.Grpc/`（高水準 facade）+ `Directory.Build.props`（netstandard2.1 + net8.0 多重 TFM）
- tier1 Go（`src/tier1/go/go.mod`）に `replace github.com/k1s0/sdk-go => ../../sdk/go` を追加。
  リリース時点 SDK が外部 registry に publish されたら `replace` を削除する運用（docs 正典）
- buf 入力 path を明示する形に `tools/codegen/buf/run.sh` を修正（workspace 全モジュール処理による
  internal 漏洩バグを併せて修正）
- TypeScript の生成 import に `.js` 拡張子を付与（`import_extension=.js`）し NodeNext 解決を満たす設定に統一

### 3. tier1 Go ファサードのハンドラ登録（DS-SW-COMP-005/006/010）— **完了**

- `src/tier1/go/cmd/{state,secret,workflow}/main.go` の `Register: nil` を `state.Register(deps)` /
  `secret.Register()` / `workflow.Register()` に置換済
- t1-state Pod は 5 API（ServiceInvoke / State / PubSub / Binding / Feature）を登録、合計 22 RPC handler を実装
  - `internal/state/{register,invoke,state,pubsub,binding,feature,context,errors}.go`
- t1-secret Pod は SecretsService（3 RPC）を登録、t1-workflow Pod は WorkflowService（6 RPC）を登録
- Dapr Go SDK adapter scaffold を `internal/adapter/dapr/{dapr,state,pubsub,binding,invoke,feature}.go` に配置
  - `Client` / 5 つの building block 別 adapter（StateAdapter / PubSubAdapter / BindingAdapter / InvokeAdapter / FeatureAdapter）
  - `ErrNotWired` センチネルで Dapr 未結線を表現、handler 側で `codes.Unimplemented` に翻訳
- 実 Dapr SDK 接続（`github.com/dapr/go-sdk/client`）と OpenBao / Temporal の結線は plan 04-04 〜 04-14 で
  順次実装。本リリース時点 では **registration shape のみ完成**（gRPC reflection で 7 service が見える）
- `go build ./...` / `go vet ./...` / `go test ./...` 全通過

### 4. tier1 Rust 3 Pod の最小実装（DS-SW-COMP-008/007/009）— **完了**

- `src/tier1/rust/Cargo.toml` workspace を配置（edition 2024、4 members: decision / audit / pii / proto-gen）
- 3 Pod crate に `Cargo.toml` + `src/main.rs` を配置:
  - `crates/decision/`（k1s0-tier1-decision）: DecisionService（Evaluate / BatchEvaluate）+ DecisionAdminService（RegisterRule / ListVersions / GetRule）登録、5 RPC 全て Status::unimplemented
  - `crates/audit/`（k1s0-tier1-audit）: AuditService（Record / Query）登録、2 RPC 全て Status::unimplemented
  - `crates/pii/`（k1s0-tier1-pii）: PiiService（Classify / Mask）登録、2 RPC 全て Status::unimplemented
- `crates/proto-gen/`（k1s0-tier1-proto-gen）: buf 生成 internal proto を `k1s0::internal::v1` module に束ねる薄い lib crate
- `k1s0-sdk-proto` を path 参照で取込（`../../sdk/rust/crates/k1s0-sdk-proto`）し公開 Service trait と型を共有
- 各 main.rs は `tokio::main` runtime + tonic Server + SIGINT/SIGTERM graceful shutdown
- `cargo metadata` / `cargo verify-project` 通過（フル `cargo check` は C リンカが必要なため CI 任せ）
- 補助 crate（common / otel-util / policy / proto）は内容実装まで `workspace.exclude` に置き、plan 04-02 / 04-08 で順次合流

### 5. infra マニフェストの実体化（IMP-DEV-POL-006）— **ほぼ完了**

- ✅ `infra/data/{cloudnativepg,kafka,minio,valkey}/` を production-grade defaults で正規化
  - CNPG: operator 3 replica + Cluster CRD（3 instance HA + WAL アーカイブ to MinIO + PodMonitor）
  - Kafka: Strimzi operator + Kafka Cluster（KRaft 3 broker + TLS mTLS + Cruise Control + JMX Exporter）
  - MinIO: distributed mode 4 replica + erasure coding + ServiceMonitor
  - Valkey: replication + Sentinel + 認証有効 + ServiceMonitor
- ✅ `infra/observability/{grafana,loki,tempo,mimir,pyroscope,otel-collector}/` を LGTM スタック production-grade で正規化
  - Loki: SimpleScalable（read/write/backend HA）+ S3（MinIO）+ ServiceMonitor + chunks/results キャッシュ
  - Tempo: distributed（compactor / distributor / ingester / querier）+ S3 + memcached
  - Mimir: distributed + S3 + zoneAwareReplication + 4 種 memcached
  - Pyroscope: micro-services + S3
  - Grafana: HA 2 replica + 外部 PG（CNPG）+ Keycloak OIDC + sidecar dashboard / datasource pickup
  - OTel Collector: agent DaemonSet（hostmetrics / filelog / OTLP receive）+ gateway Deployment 3 replica（tail_sampling + 4 backend ファンアウト）
- ✅ `infra/mesh/istio-ambient/` を Ambient Mesh production-grade で正規化
  - istiod HA 3 replica + HPA（CPU 70%、3〜10）+ OTel tracing 連携 + ztunnel sizing
  - mTLS STRICT 強制 PeerAuthentication
- ✅ `infra/security/{cert-manager,keycloak,spire,kyverno}/` を正規化
  - cert-manager: HA 3 + Let's Encrypt prod/staging + 内部 CA ClusterIssuer + Gateway API integration
  - Keycloak: HA 3 + Infinispan + 外部 CNPG + production mode + realm import + ServiceMonitor
  - SPIRE: server HA 3 + 外部 CNPG dataStore + cert-manager upstream CA + CSI driver + OIDC Discovery
  - Kyverno: admission HA 3 + 4 baseline ClusterPolicy（runAsNonRoot / privileged 禁止 / k1s0.io/component label / resource requests）
- ✅ `infra/k8s/{bootstrap,namespaces,networking,storage}/` を正規化
  - bootstrap: Cluster API + KubeadmControlPlane HA 3 / kubeadm-init 代替
  - namespaces: 17 layer Pod Security Standards label + Istio Ambient mode label
  - networking: Calico VXLAN + MetalLB（IPAddressPool は環境別 overlay）
  - storage: 4 種 StorageClass（default / high-iops / shared / backup）
- ✅ `infra/scaling/keda/`、`infra/feature-management/flagd/`、`infra/dapr/control-plane/`、`infra/security/openbao/` を正規化
  - KEDA: operator HA 2 + metrics-apiserver 2 + admission webhook + ServiceMonitor
  - flagd: HA 3 + ConfigMap baseline flag + ServiceMonitor
  - Dapr control-plane: 全 component HA 3 + mTLS + Raft（placement）
  - OpenBao: HA 3 + Raft integrated storage + cert-manager TLS + Vault Agent Injector
- 🔲 残り（plan 05-XX 以降）:
  - `infra/environments/{dev,staging,prod}/` overlay 雛形（環境別パラメータ上書き）
  - `infra/mesh/envoy-gateway/`（Istio Ambient と相補、特定 use case のみ）
  - 各 component の上流 Helm chart version pin の Renovate 追従
  - production CSI provisioner の確定（PLACEHOLDER_csi_provisioner の置換）

### 6. deploy 拡充（IMP-REL-* / ADR-CICD-*）— **大半完了**

- ✅ `deploy/charts/` の残り 5 chart を tier1-facade 同水準で配置
  - tier1-rust-service: Rust 3 Pod（decision/audit/pii）ループ生成、distroless/cc
  - tier2-go-service: 汎用 Go テンプレート（Dapr sidecar / HPA / probe httpGet /healthz /readyz）
  - tier2-dotnet-service: 汎用 .NET テンプレート（aspnet runtime UID 1654 / ASPNETCORE_URLS / DOTNET_ENVIRONMENT）
  - tier3-bff: Go BFF テンプレート（OIDC / ingress / Dapr sidecar）
  - tier3-web-app: nginx + SPA fallback + `/api/` reverse proxy to BFF
  - `helm lint` 全 5 chart 通過、`helm template` 描画確認済
- ✅ `deploy/rollouts/{canary-strategies,analysis-templates}/` に Argo Rollouts CRD
  - canary-25-50-100: 3 段階 setWeight + pause + analysis（自動評価）
  - error-rate / latency-p99: Mimir（Prometheus 互換）クエリベースの自動評価
- ✅ `deploy/kustomize/{base,overlays/{dev,staging,prod}}/` に 6 chart × 3 環境の values overlay + base 共通リソース + README を配置
  - dev: replica=1 / debug log / 最小 resources
  - staging: replica=2 / info log / HPA 緩い
  - prod: replica=3 / warn log / podAntiAffinity / image semver pinning / HPA 実測ベース
- ✅ `deploy/image-updater/` Argo CD Image Updater 設定（HA 2 + GHCR registry + Git write-back）
- 🔲 残り（採用後の運用拡大時）:
  - 各 chart の Deployment → Rollout への置換（Argo Rollouts CRD 適用）
  - ApplicationSet への image-updater annotation 注入

### 7. examples 完動 4 種（IMP-DIR-COMM-113）— **完了**

- ✅ tier2-go-service: cmd + go.mod + Dockerfile + catalog-info、k1s0 SDK の `c.State().Save()` を HTTP
  POST /sample-write で呼ぶデモ（`go build ./...` 通過）
- ✅ tier2-dotnet-service: ASP.NET Core minimal API + .csproj + appsettings + Dockerfile + catalog-info、
  `client.State.SaveAsync()` を HTTP POST /sample-write で呼ぶ（`dotnet build` 通過）
- ✅ tier3-bff-graphql: cmd + go.mod + Dockerfile + catalog-info、POST /graphql で GraphQL クエリ
  `stateGet(store, key)` を受けて State API を呼ぶ最小 BFF（`go build ./...` 通過）
- ✅ tier3-native-maui: .csproj + PortalViewModel + catalog-info、MVVM パターンで
  `client.State.GetAsync()` をバインディング経由で呼ぶ最小 ViewModel（`dotnet build` 通過、
  リリース時点 は net8.0 単独、採用初期で android/ios/maccatalyst/windows multi-target に拡張）
- ✅ 週次 E2E workflow を 7 種配置（`.github/workflows/example-{tier1-rust-service,tier1-go-facade,
  tier2-go-service,tier2-dotnet-service,tier3-web-portal,tier3-bff-graphql,tier3-native-maui}.yml`）。
  cron `0 3 * * 1`（毎週月曜 UTC 03:00 = JST 12:00）+ 関連 path の PR 変更時 + workflow_dispatch
- 🔲 残り（plan 06-XX 以降）:
  - 各 example の unit / integration test 追加（テスト雛形は配置済の規約に沿って）
  - tier3-native-maui の Android / iOS multi-target ビルド

### 8. SDK 高水準ファサード（README に示された動詞統一）— **完了**

- ✅ 4 言語すべてに Client + 動詞統一 facade を 12 service 全件で配置
  - Go: `src/sdk/go/k1s0/{client,state,pubsub,secrets,log,workflow,decision,audit,pii,feature,binding,invoke,telemetry,doc}.go`
  - Rust: `src/sdk/rust/crates/k1s0-sdk/src/{lib,client,state,pubsub,secrets,log,workflow,decision,audit,pii,feature,binding,invoke,telemetry}.rs`
  - TypeScript: `src/sdk/typescript/src/{index,client,state,pubsub,secrets,log,workflow,decision,audit,pii,feature,binding,invoke,telemetry}.ts`
  - .NET: `src/sdk/dotnet/src/K1s0.Sdk.Grpc/{K1s0Client,StateFacade,PubSubFacade,SecretsFacade,LogFacade,WorkflowFacade,DecisionFacade,AuditFacade,PiiFacade,FeatureFacade,BindingFacade,InvokeFacade,TelemetryFacade}.cs`
- ビルド検証: `go build ./...` / `cargo verify-project` / `pnpm build` / `dotnet build Sdk.sln` 全通過
- 動詞統一 API（README コードサンプル整合）:
  - State: Get / Save / Delete  Log: Send / Info / Warn / Error / Debug
  - PubSub: Publish              Telemetry: EmitMetric / EmitSpan
  - Secrets: Get / Rotate        Audit: Record / Query
  - Workflow: Start / Signal / Query / Cancel / Terminate / GetStatus
  - Decision: Evaluate / BatchEvaluate
  - Pii: Classify / Mask          Feature: EvaluateBoolean/String/Number/Object
  - Binding: Invoke               ServiceInvoke: Call
- ✅ Admin service facade（DecisionAdmin / FeatureAdmin）も 4 言語で同梱（14 service 全件）
- ✅ Stream RPC facade（InvokeStream / PubSub.Subscribe）も 4 言語で同梱
  - Go: `c.Invoke().Stream(ctx, ..., handler)` / `c.PubSub().Subscribe(ctx, ..., handler)`（callback ベース）
  - Rust: `client.invoke().stream(...).await` / `client.pubsub().subscribe(...).await`（`tonic::Streaming<T>` 返却）
  - TypeScript: `client.invoke.stream(...)` / `client.pubsub.subscribe(...)`（`AsyncIterable<T>` 返却、`for await` で消費）
  - .NET: `client.Invoke.StreamAsync(...)` / `client.PubSub.SubscribeAsync(...)`（`IAsyncEnumerable<T>` 返却、`await foreach` で消費）
- ✅ 各 facade の単体テスト雛形を 4 言語に配置
  - Go: `src/sdk/go/k1s0/client_test.go`（bufconn ベース、4 tests）
  - Rust: `src/sdk/rust/crates/{k1s0-sdk,k1s0-sdk-proto}/tests/smoke.rs`（Config / proto 型 / Severity / K1s0ErrorCategory 整合）
  - TypeScript: `src/sdk/typescript/src/__tests__/client.test.ts` + `vitest.config.ts`（5 tests / vitest）
  - .NET: `src/sdk/dotnet/tests/K1s0.Sdk.Grpc.Tests/`（xunit + coverlet、5 tests）
- 🔲 残り（採用後の運用拡大時）:
  - netstandard2.1 多重 TFM 再導入（OSS 配布の互換性向上）

各タスクは完成のたびに本 SHIP_STATUS.md のマチュリティ表を更新する運用とする。
