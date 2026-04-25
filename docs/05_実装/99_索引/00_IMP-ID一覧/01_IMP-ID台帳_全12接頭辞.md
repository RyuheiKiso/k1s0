# 99. 索引 / 00. IMP-ID 一覧 / 01. IMP-ID 台帳 全 12 接頭辞

本ファイルは `05_実装/` 12 章が独立に採番する IMP-\* ID の権威台帳である。IMP-BUILD / IMP-CODEGEN / IMP-CI / IMP-DEP / IMP-DEV / IMP-OBS / IMP-REL / IMP-SUP / IMP-SEC / IMP-POL / IMP-DX / IMP-TRACE の 12 接頭辞について、採番済 ID の概要・所属ファイル・適用段階を章接頭辞ごとにまとめる。00 章 `00_ディレクトリ設計/` の IMP-DIR-\* は `00_ディレクトリ設計/90_トレーサビリティ/01_IMP-DIR_ID一覧.md` で自律管理されるため、本章の台帳には再掲せず章末で相互参照のみ張る。

## 台帳の読み方

台帳は接頭辞ごとに 1 節を配し、節冒頭で「何を採番しているか / どの原則ファイルで方針が確定しているか / 採番済件数」を文で示し、続いて採番済 ID の表、最後に予約帯の残量を付す。表の 1 行は 1 つの設計判断を指し、IMP-TRACE-POL-003（1 判断 = 1 ID 原子性）の系として、異なる判断を束ねた ID は存在しない前提で読んでよい。

ID の所属ファイル列は `05_実装/` からの相対パスで表記する（例: `10_ビルド設計/10_Rust_Cargo_workspace/01_Rust_Cargo_workspace.md`）。適用段階列は当該 ID が **初めて効力を持つ** 段階を示し、k1s0 OSS リリース時点で同梱されるが効力発生が採用側組織の運用習熟度に依存する ID（例: sccache のリモートキャッシュ運用）は適用段階を別記する。予約中 ID については範囲のみを示し、採番時に行を追加する運用とする（IMP-TRACE-POL-004 に基づき本章が最終更新先となる）。

## 接頭辞サマリ

接頭辞ごとの採番状況を先に俯瞰できるよう、サマリ表を以下に示す。各接頭辞は 001〜099 の 99 枠を持ち（IMP-TRACE-POL-002）、POL（原則）・サブ接頭辞別実装 ID の 2 段構成を基本とする。POL は各章 7 件固定で、サブ接頭辞は章ごとに 1〜4 種類が採番されている。

| 接頭辞 | 章 | POL | 実装 ID | 採番済合計 | 予約残 | 主サブ接頭辞 |
|---|---|---|---|---|---|---|
| IMP-BUILD | 10 ビルド | 7 | 15 | 22 | 77 | CW（Rust workspace）/ GM（Go module） |
| IMP-CODEGEN | 20 コード生成 | 7 | 16 | 23 | 76 | BUF（buf 生成）/ SCF（Scaffold CLI） |
| IMP-CI | 30 CI/CD | 7 | 24 | 31 | 68 | RWF（reusable workflow）/ HAR（Harbor/Trivy） |
| IMP-DEP | 40 依存管理 | 7 | 26 | 33 | 66 | REN（Renovate 中央運用）/ SBM（SBOM 差分監視）/ LIC（ライセンス判定） |
| IMP-DEV | 50 開発者体験 | 7 | 15 | 22 | 77 | DC（Dev Container）/ GP（Golden Path） |
| IMP-OBS | 60 観測性 | 7 | 29 | 36 | 63 | OTEL / SLO / SLI / INC |
| IMP-REL | 70 リリース | 7 | 18 | 25 | 74 | ARG（ArgoCD）/ PD（Progressive Delivery） |
| IMP-SUP | 80 サプライチェーン | 7 | 18 | 25 | 74 | COS（cosign）/ FOR（Forensics） |
| IMP-SEC | 85 Identity | 7 | 38 | 45 | 54 | KC（Keycloak）/ SP（SPIRE）/ REV（退職 revoke）/ KEY |
| IMP-POL | 90 ガバナンス | 7 | 37 | 44 | 55 | KYV（Kyverno）/ ADR（ADR プロセス）/ TR（Technology Radar）/ STR（STRIDE） |
| IMP-DX | 95 DX メトリクス | 7 | 11 | 18 | 81 | DORA（DORA Four Keys） |
| IMP-TRACE | 99 索引 | 7 | 0 | 7 | 92 | 未採番（POL のみ。索引は採用初期から採番） |

リリース時点 全体で採番済 ID は計 331 件（POL 84 件 + 実装 247 件）、予約残は 857 件。実装 247 件は リリース時点 段階の「核心節 19 ファイル + 依存管理 3 節 + ガバナンス 4 節」から抽出した純粋な採番で、POL を除く。

## IMP-BUILD（10 章 ビルド設計）

4 言語並行ビルド（Cargo workspace / go.mod / pnpm workspace / dotnet sln）の設計判断を採番する。方針 7 件は `10_ビルド設計/00_方針/01_ビルド設計原則.md` で確定、実装 ID は Rust Cargo workspace と Go module 分離の 2 節に割り当てられている。運用蓄積後は pnpm workspace / dotnet sln の追加節で CW / GM と並列のサブ接頭辞（PW / DN）が採番される想定である。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-BUILD-POL-001 | 言語ネイティブビルド優先（Bazel 不採用） | `10_ビルド設計/00_方針/01_ビルド設計原則.md` | 0 |
| IMP-BUILD-POL-002 | ワークスペース境界 = tier 境界 | 同上 | 0 |
| IMP-BUILD-POL-003 | 依存方向逆流の lint 拒否 | 同上 | 0 |
| IMP-BUILD-POL-004 | path-filter による選択ビルド | 同上 | 0 |
| IMP-BUILD-POL-005 | 3 層キャッシュ階層 | 同上 | 採用初期 |
| IMP-BUILD-POL-006 | ビルド時間 SLI 計測 | 同上 | 0 |
| IMP-BUILD-POL-007 | 生成物 commit と隔離 | 同上 | 0 |
| IMP-BUILD-CW-010 | tier1 Rust と SDK Rust の 2 workspace 分割 | `10_ビルド設計/10_Rust_Cargo_workspace/01_Rust_Cargo_workspace.md` | 0 |
| IMP-BUILD-CW-011 | `[workspace.dependencies]` による外部 crate 集約 | 同上 | 0 |
| IMP-BUILD-CW-012 | rust-toolchain.toml による stable 固定と edition 2024 統一 | 同上 | 0 |
| IMP-BUILD-CW-013 | sccache ローカル + CI リモートの 2 層キャッシュ | 同上 | 採用初期 |
| IMP-BUILD-CW-014 | cargo-deny による license / ban / advisory 強制 | 同上 | 0 |
| IMP-BUILD-CW-015 | clippy `-D warnings` + unwrap / panic 独自 deny lint | 同上 | 0 |
| IMP-BUILD-CW-016 | cargo nextest による並列テストとタイムアウト強制 | 同上 | 0 |
| IMP-BUILD-CW-017 | tier1 Rust の unsafe 全面禁止と SDK Rust unwrap 警告緩和 | 同上 | 0 |
| IMP-BUILD-GM-020 | 5 module 分離（tier1 Go / tier2 Go / BFF / SDK Go / tests） | `10_ビルド設計/20_Go_module分離戦略/01_Go_module分離戦略.md` | 0 |
| IMP-BUILD-GM-021 | 内部 module 参照の `replace` ディレクティブ適用 | 同上 | 0 |
| IMP-BUILD-GM-022 | 独自 linter による依存方向逆流検出 | 同上 | 0 |
| IMP-BUILD-GM-023 | `go mod tidy -diff` による go.sum ドリフト検出 | 同上 | 0 |
| IMP-BUILD-GM-024 | path-filter 選択ビルドの Go 面実装 | 同上 | 0 |
| IMP-BUILD-GM-025 | GOCACHE の actions/cache リモート化 | 同上 | 採用初期 |
| IMP-BUILD-GM-026 | `go.work` 不採用（commit 事故防止） | 同上 | 0 |
| IMP-BUILD-GM-027 | SDK Go module name の外部公開経路固定 | 同上 | 0 |

予約: IMP-BUILD-CW-018〜019 / IMP-BUILD-GM-028〜029 / IMP-BUILD-028〜099（pnpm workspace / dotnet sln の リリース時点 展開用）は予約中。

## IMP-CODEGEN（20 章 コード生成設計）

Protobuf / OpenAPI からの生成と Scaffold CLI による雛形生成の判断を採番する。方針は `20_コード生成設計/00_方針/01_コード生成原則.md`、実装は BUF（buf 生成パイプライン）と SCF（Scaffold CLI）の 2 節に分割される。リリース時点 で Backstage Software Template 統合の ID（SCF 系）が追加採番される予定。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-CODEGEN-POL-001〜007 | 契約単一真実源 / 生成機械化 / buf breaking / DO NOT EDIT / golden snapshot / catalog-info.yaml 必須 / テンプレート変更二重承認 | `20_コード生成設計/00_方針/01_コード生成原則.md` | 0 |
| IMP-CODEGEN-BUF-010 | 単一 `buf.yaml` + 言語別 `buf.gen.*.yaml` 4 分割 | `20_コード生成設計/10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md` | 0 |
| IMP-CODEGEN-BUF-011 | tier1 サーバと SDK の生成先物理パス分離 | 同上 | 0 |
| IMP-CODEGEN-BUF-012 | `include_types` による internal package の SDK 除外 | 同上 | 0 |
| IMP-CODEGEN-BUF-013 | `buf breaking` FILE レベルの必須ゲート | 同上 | 0 |
| IMP-CODEGEN-BUF-014 | 生成 drift 検出スクリプトによる DO NOT EDIT 強制 | 同上 | 0 |
| IMP-CODEGEN-BUF-015 | `tools/codegen/buf.version` による CLI バージョン固定 | 同上 | 0 |
| IMP-CODEGEN-BUF-016 | v1 → v2 ディレクトリ分岐による breaking 変更経路 | 同上 | 採用初期 |
| IMP-CODEGEN-BUF-017 | `.gitattributes` の linguist-generated 宣言 | 同上 | 0 |
| IMP-CODEGEN-SCF-030 | Rust 実装の Scaffold CLI（`src/platform/scaffold/`） | `20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md` | 0 |
| IMP-CODEGEN-SCF-031 | Backstage Software Template 互換の `template.yaml` 採用 | 同上 | 採用初期 |
| IMP-CODEGEN-SCF-032 | tier2 / tier3 テンプレート配置分離 | 同上 | 0 |
| IMP-CODEGEN-SCF-033 | `catalog-info.yaml` の自動生成と CODEOWNERS 連動 | 同上 | 0 |
| IMP-CODEGEN-SCF-034 | SRE + Security 二重承認の branch protection 強制 | 同上 | 0 |
| IMP-CODEGEN-SCF-035 | golden snapshot 検証と `UPDATE_GOLDEN=1` 承認プロセス | 同上 | 0 |
| IMP-CODEGEN-SCF-036 | テンプレート semver バージョニング | 同上 | 0 |
| IMP-CODEGEN-SCF-037 | リリース時点 の Backstage UI 統合経路（custom action 化） | 同上 | 採用初期 |

予約: IMP-CODEGEN-BUF-018〜019 / IMP-CODEGEN-SCF-038〜039 / IMP-CODEGEN-040〜099（リリース時点 の OpenAPI 生成 / 1b の Backstage 統合用）。

## IMP-CI（30 章 CI/CD 設計）

7 段階 CI（lint → test → build → push → sign → deploy → verify）と Harbor/Trivy/cosign の統合判断を採番する。方針は `30_CI_CD設計/00_方針/01_CI_CD原則.md`、実装は RWF（reusable workflow）と HAR（Harbor/Trivy push）の 2 節。リリース時点 で OPA/Kyverno ポリシー段（POL サブ接頭辞）が追加予定。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-CI-POL-001〜007 | Harbor push 完結 / quality gate 統制 / path-filter / Trivy Critical 拒否 / cosign keyless 署名 / branch protection / Renovate 自動化 | `30_CI_CD設計/00_方針/01_CI_CD原則.md` | 0 |
| IMP-CI-RWF-010 | reusable workflow 4 本（lint / test / build / push） | `30_CI_CD設計/10_reusable_workflow/01_reusable_workflow設計.md` | 0 |
| IMP-CI-RWF-011 | docs 単独変更時の lint-only 経路 | 同上 | 0 |
| IMP-CI-RWF-012 | path-filter golden test による filter 変更保護 | 同上 | 0 |
| IMP-CI-RWF-013 | Karpenter + ARC による runner 自動スケール | 同上 | 採用初期 |
| IMP-CI-RWF-014 | CI secret の最小集合化と注入経路固定 | 同上 | 0 |
| IMP-CI-RWF-015 | env 明示・secret echo 禁止の lint | 同上 | 0 |
| IMP-CI-RWF-016 | cache キー規約と共有 backend | 同上 | 0 |
| IMP-CI-RWF-017 | reusable workflow の tag 固定参照と Renovate 連携 | 同上 | 0 |
| IMP-CI-RWF-018 | coverage 閾値の段階導入（0 計測のみ → 1a 80% → 1b 90% → 2 mutation） | 同上 | 0 |
| IMP-CI-RWF-019 | 失敗時の可読性（failure_reason outputs と Loki / Mimir 連携） | 同上 | 採用初期 |
| IMP-CI-RWF-020 | workflow リポジトリ分離の リリース時点 不採用と リリース時点 再評価 | 同上 | 0 |
| IMP-CI-RWF-021 | composite action の内部実装扱いと `tools/ci/actions` 配置 | 同上 | 0 |
| IMP-CI-HAR-040 | Harbor 物理配置と CloudNativePG バックエンド | `30_CI_CD設計/40_Harbor_Trivy_push/01_Harbor_Trivy_push設計.md` | 0 |
| IMP-CI-HAR-041 | 5 Harbor project（tier1 / tier2 / tier3 / infra / sdk）と RBAC 分離 | 同上 | 0 |
| IMP-CI-HAR-042 | quarantine プロジェクトへの自動隔離 | 同上 | 採用初期 |
| IMP-CI-HAR-043 | robot アカウントの 60 日自動ローテ | 同上 | 0 |
| IMP-CI-HAR-044 | CVSS 連動の 4 段階閾値運用 | 同上 | 0 |
| IMP-CI-HAR-045 | Trivy DB の日次更新とオフラインミラー（リリース時点） | 同上 | 採用初期 |
| IMP-CI-HAR-046 | allowlist 例外の 30 日時限 + Security 承認 | 同上 | 0 |
| IMP-CI-HAR-047 | cosign keyless 署名と Rekor 記録 | 同上 | 0 |
| IMP-CI-HAR-048 | Harbor DR replication と 段階展開 | 同上 | 採用初期 |
| IMP-CI-HAR-049 | Harbor / Trivy / cosign の SLI 計測と SLO 定義 | 同上 | 0 |
| IMP-CI-HAR-050 | 手動 push 禁止と緊急時の一時付与手順 | 同上 | 0 |
| IMP-CI-HAR-051 | Retention / GC policy とスナップショット管理 | 同上 | 0 |

予約: IMP-CI-RWF-022〜029 / IMP-CI-HAR-052〜059 / IMP-CI-060〜099（リリース時点: POL サブ接頭辞、ポリシー段追加用）。

## IMP-DEP（40 章 依存管理設計）

Renovate 中央集約・lockfile commit 必須・SBOM 添付の 7 原則に加え、リリース時点 核心節で REN（Renovate 中央運用）/ SBM（SBOM 差分監視）/ LIC（ライセンス判定）の 3 サブ接頭辞を展開済。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-DEP-POL-001〜007 | Renovate 経由のみ / lockfile commit 必須 / vendoring UPSTREAM 必須 / SPDX 表示と BUSL/SSPL 自動拒否 / AGPL 6 件の分離境界恒常検証 / 自動マージは patch のみ / SBOM 全アーティファクト添付 | `40_依存管理設計/00_方針/01_依存管理原則.md` | 0 |
| IMP-DEP-REN-010〜019 | Renovate 中央 preset / preset バージョン固定 / cron 実行時刻統制 / Dependency Dashboard 有効化 / automerge 条件 / 15 package manager 横断 ほか | `40_依存管理設計/10_Renovate中央運用/01_Renovate中央運用.md` | リリース時点〜採用初期 |
| IMP-DEP-SBM-020〜027 | CycloneDX 形式統一 / syft による SBOM 生成 / diff スクリプト / CI ゲート / Grype 連動 / 脆弱性通知ワークフロー ほか | `40_依存管理設計/20_SBOM差分監視/01_SBOM差分監視.md` | リリース時点〜採用初期 |
| IMP-DEP-LIC-030〜037 | SPDX ライセンス分類 / BUSL/SSPL 即時拒否 / AGPL 6 件の分離検証 / GitHub Action gate / third_party UPSTREAM.md 強制 ほか | `40_依存管理設計/30_ライセンス判定/01_ライセンス判定.md` | リリース時点〜採用初期 |

予約: IMP-DEP-038〜099（運用蓄積後で CDN / CI キャッシュ整合 / 言語別 lockfile 検証など追加予定）。

## IMP-DEV（50 章 開発者体験設計）

Paved Road 思想・10 役 Dev Container・Golden Path examples の 3 本柱を採番する。方針は `50_開発者体験設計/00_方針/01_開発者体験原則.md`、実装は DC（Dev Container）と GP（Golden Path）の 2 節。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-DEV-POL-001〜007 | Paved Road 一本化 / Scaffold 経由 / 10 役 Dev Container / time-to-first-commit SLI / IDE 設定共有資産 / kind/k3d + Dapr Local / Scaffold 変更の SRE+Security 二重承認 | `50_開発者体験設計/00_方針/01_開発者体験原則.md` | 0 |
| IMP-DEV-DC-010 | `.devcontainer/` と `tools/devcontainer/profiles/` の二層構造 | `50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md` | 0 |
| IMP-DEV-DC-011 | 10 役別ベースイメージとサイズ目標（docs-writer 1GB / full 8GB） | 同上 | 0 |
| IMP-DEV-DC-012 | `devcontainer.json` 共通パターン（features / extensions / settings / postCreate） | 同上 | 0 |
| IMP-DEV-DC-013 | VS Code 設定分離（common + role） | 同上 | 0 |
| IMP-DEV-DC-014 | ローカル Kubernetes（kind / k3d）と Dapr Local 統合 | 同上 | 0 |
| IMP-DEV-DC-015 | OpenBao dev server のローカル展開 | 同上 | 採用初期 |
| IMP-DEV-DC-016 | Dev Container image の digest 固定と Renovate 連動 | 同上 | 0 |
| IMP-DEV-DC-017 | time-to-first-commit 計測点の露出 | 同上 | 0 |
| IMP-DEV-GP-020 | `examples/` 配下の 4 つの リリース時点 必須 example 配置 | `50_開発者体験設計/20_Golden_Path_examples/01_Golden_Path_examples.md` | 0 |
| IMP-DEV-GP-021 | `catalog-info.yaml` 同梱による Backstage Examples カタログ自動登録 | 同上 | 採用初期 |
| IMP-DEV-GP-022 | Dapr components.yaml 同梱とローカル起動（`make up` 一発） | 同上 | 0 |
| IMP-DEV-GP-023 | README の 5 セクション必須化 | 同上 | 0 |
| IMP-DEV-GP-024 | PR / nightly / 月次 e2e の 3 層 CI 検証 | 同上 | 0 |
| IMP-DEV-GP-025 | リリース時点 で 8 例への拡大 | 同上 | 採用初期 |
| IMP-DEV-GP-026 | example の所有権（`@k1s0/platform-dx` 恒久保有）と lifecycle: experimental 明示 | 同上 | 0 |

予約: IMP-DEV-DC-018〜019 / IMP-DEV-GP-027〜029 / IMP-DEV-030〜099（リリース時点: onboarding 自動化 / Scorecards 連携用）。

## IMP-OBS（60 章 観測性設計）

OTel Collector 配置・SLO/SLI 定義・Incident Taxonomy の 3 本柱を採番する。方針は `60_観測性設計/00_方針/01_観測性原則.md`、実装は OTEL（Collector 配置）/ SLO（SLO/SLI 定義）/ INC（Incident Taxonomy）の 3 節。SLI 派生の SLO-003 が存在するのは、リリース時点 の時点で p99 ベース SLI と SLO の関係を個別採番しているため。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-OBS-POL-001〜007 | OTel Collector 集約 / LGTM AGPL 分離維持 / Google SRE Book 準拠 / Incident Taxonomy 可用性×セキュリティ統合 / Error Budget 100% 消費時 feature 凍結 / Runbook と SLI 紐付け / DORA 4 keys は 95 章 | `60_観測性設計/00_方針/01_観測性原則.md` | 0 |
| IMP-OBS-OTEL-010〜019 | Agent/Gateway 二層、tail sampling、PII transform、AGPL 境界など OTel Collector 10 ID | `60_観測性設計/10_OTel_Collector配置/01_OTel_Collector配置.md` | リリース時点〜採用初期 |
| IMP-OBS-SLO-040〜047 | tier1 公開 11 API の p99 SLO 階層、Runbook 一対一対応、外部 SLA 99% との乖離許容など | `60_観測性設計/40_SLO_SLI定義/01_tier1_公開11API_SLO_SLI.md` | 0 |
| IMP-OBS-SLI-003 | Availability SLI 初期定義（リリース時点 ブートストラップ）| `60_観測性設計/40_SLO_SLI定義/01_tier1_公開11API_SLO_SLI.md` | 0 |
| IMP-OBS-INC-060〜071 | Incident Taxonomy の AVL / SEC 統合、Sev1/Sev2 × Runbook 4 セル、PagerDuty 連動、NFR-E-SIR-002 72 時間通告など 12 ID | `60_観測性設計/60_Incident_Taxonomy/01_Incident_Taxonomy統合分類.md` | 0 |

予約: IMP-OBS-OTEL-020〜029 / IMP-OBS-SLO-048〜059 / IMP-OBS-INC-072〜079 / IMP-OBS-080〜099（リリース時点: Tempo トレース拡張 / Pyroscope 合流）。

## IMP-REL（70 章 リリース設計）

ArgoCD による GitOps と Argo Rollouts による Progressive Delivery の判断を採番する。方針は `70_リリース設計/00_方針/01_リリース原則.md`、実装は ARG（ArgoCD App 構造）/ PD（Progressive Delivery / AnalysisTemplate）の 2 節。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-REL-POL-001〜007 | GitOps 一本化 / Progressive Delivery 必須 / Canary AnalysisTemplate 強制 / 手動 rollback 15 分以内 / Rollback 経路単一化 / flag 即時切替分離 / App-of-Apps 構造 | `70_リリース設計/00_方針/01_リリース原則.md` | 0 |
| IMP-REL-ARG-010〜017 | App-of-Apps、ApplicationSet、sync policy、環境別 overlays など ArgoCD 8 ID | `70_リリース設計/10_ArgoCD_App構造/01_ArgoCD_App構造.md` | リリース時点〜採用初期 |
| IMP-REL-PD-020〜028 | Canary 段階（5/25/50/100）、AnalysisTemplate、failureLimit、手動 rollback、CR 定義など Argo Rollouts 9 ID | `70_リリース設計/20_ArgoRollouts_PD/01_ArgoRollouts_PD設計.md` | リリース時点〜採用初期 |

予約: IMP-REL-ARG-018〜019 / IMP-REL-PD-029〜039 / IMP-REL-040〜099（リリース時点: 環境パイプライン拡張 / 多リージョン、採用後の運用拡大時: マルチクラスタ）。

## IMP-SUP（80 章 サプライチェーン設計）

cosign keyless 署名と Forensics Runbook の判断を採番する。方針は `80_サプライチェーン設計/00_方針/01_サプライチェーン原則.md`、実装は COS（cosign）/ FOR（Forensics Runbook）の 2 節。リリース時点 で SBOM / SLSA Provenance 節（SBM / SLSA サブ接頭辞）が追加予定。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-SUP-POL-001〜007 | SLSA L2 先行 → L3 到達 / cosign keyless 必須 / SBOM 全アーティファクト添付 / Rekor 記録永続 / Forensics Runbook 起点 image hash / Kyverno 署名検証 / 鍵ライフサイクル管理 | `80_サプライチェーン設計/00_方針/01_サプライチェーン原則.md` | リリース時点〜採用初期 |
| IMP-SUP-COS-010〜018 | cosign keyless 9 ID（OIDC 発行、Rekor 記録、検証経路、鍵なし運用の可監査性、攻撃面） | `80_サプライチェーン設計/10_cosign署名/01_cosign_keyless署名.md` | 0 |
| IMP-SUP-FOR-040〜048 | Forensics Runbook 9 ID（image hash → 影響範囲、SBOM 突合、48 時間 SLA、Kyverno 迂回検知、PagerDuty エスカレ） | `80_サプライチェーン設計/40_Forensics_Runbook/01_image_hash逆引き_Forensics.md` | リリース時点〜採用初期 |

予約: IMP-SUP-COS-019〜029 / IMP-SUP-FOR-049〜059 / IMP-SUP-020〜039（SBOM）/ IMP-SUP-060〜099（SLSA L3 リリース時点 用）。

## IMP-SEC（85 章 Identity 設計）

Keycloak / SPIRE / OpenBao / cert-manager / 退職 revoke / Istio mTLS の 6 原則と実装を採番する。他章と異なり実装 ID が突出して多いのは、Identity が人間 ID とワークロード ID の 2 系統を抱えるため。方針は `85_Identity設計/00_方針/01_Identity原則.md`。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-SEC-POL-001〜007 | 人間 ID Keycloak 集約 / ワークロード ID SPIRE / 退職 revoke 60 分以内 / OpenBao Secret 集約 / cert-manager 証明書自動更新 / Istio Ambient mTLS / GameDay 継続検証 | `85_Identity設計/00_方針/01_Identity原則.md` | 0 |
| IMP-SEC-KEY-001〜002 | 鍵管理の初期採番（リリース時点 の OpenBao KV-v2 / PKI 展開向け） | 同上 | 採用初期 |
| IMP-SEC-KC-010〜022 | Keycloak realm 13 ID（tenant 分離、JWT claims、admin event、login flow、MFA、2FA、7 年監査保存） | `85_Identity設計/10_Keycloak_realm/01_Keycloak_realm設計.md` | 0 |
| IMP-SEC-SP-020〜035 | SPIRE/SPIFFE 16 ID（HA 3 replica、DaemonSet、PSAT 認証、SVID 発行、Dapr 統合、Istio 統合、GameDay） | `85_Identity設計/20_SPIRE_SPIFFE/01_SPIRE_SPIFFE設計.md` | リリース時点〜採用初期 |
| IMP-SEC-REV-050〜059 | 退職 revoke Runbook 10 ID（起点通知、60 分 SLA、7 年監査ログ、Object Lock、Service Account 最小権限） | `85_Identity設計/50_退職時revoke手順/01_退職時revoke手順.md` | 0 |

予約: IMP-SEC-KC-023〜029 / IMP-SEC-SP-036〜049 / IMP-SEC-REV-060〜069 / IMP-SEC-070〜099（リリース時点: OpenBao KV-v2 / PKI / Transit / Keycloak SCIM 連携）。

## IMP-POL（90 章 ガバナンス設計）

Kyverno による dual ownership（Platform + Security）ポリシー 7 原則に加え、リリース時点 核心節で KYV（Kyverno 詳細）/ ADR（ADR 運用プロセス）/ TR（Technology Radar）/ STR（STRIDE 脅威モデル）の 4 サブ接頭辞を展開済。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-POL-POL-001〜007 | Kyverno dual ownership / audit 2 週間前置 / 例外 30 日時限 / 脅威モデル ADR 化 / Runbook 紐付け / WORM 監査 / NetworkPolicy 2 層 | `90_ガバナンス設計/00_方針/01_ガバナンス原則.md` | 0 |
| IMP-POL-KYV-010〜022 | validate/mutate/generate Policy 分類 / exception 30 日時限 / audit → enforce 段階移行 / Policy Report 集約 ほか | `90_ガバナンス設計/10_Kyverno_Policy/01_Kyverno_Policy設計.md` | リリース時点〜採用初期 |
| IMP-POL-ADR-020〜027 | ADR ライフサイクル / Proposed → Accepted 2 週間審査 / Superseded 明示 / リンク双方向 ほか | `90_ガバナンス設計/20_ADR_プロセス/01_ADR運用プロセス設計.md` | リリース時点〜採用初期 |
| IMP-POL-TR-030〜037 | Technology Radar 四象限 / Assess → Trial → Adopt 移行基準 / Hold 条件 / Scaffold CLI 連動 ほか | `90_ガバナンス設計/30_Technology_Radar/01_Technology_Radar運用設計.md` | リリース時点〜採用初期 |
| IMP-POL-STR-040〜047 | STRIDE 脅威モデル / 11 API マトリクス / 緩和策 ADR 化 / 再評価 trigger ほか | `90_ガバナンス設計/40_脅威モデル_STRIDE/01_STRIDE脅威モデル.md` | リリース時点〜採用初期 |

予約: IMP-POL-048〜099（リリース時点: OPA Gatekeeper 連携 / Cilium NetworkPolicy 展開 / SLSA ポリシー統合）。

## IMP-DX（95 章 DX メトリクス）

DORA Four Keys の計測設計を採番する。方針は `95_DXメトリクス/00_方針/01_DXメトリクス原則.md`、実装は DORA（Four Keys 計測）節。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-DX-POL-001〜007 | DORA Four Keys を リリース時点 計測 / Severity 別分離 / Deploy Frequency 分母定義 / MTTR ユーザ影響終点 / time-to-first-commit 独自 SLI / Backstage Scorecards / 四半期レビュー | `95_DXメトリクス/00_方針/01_DXメトリクス原則.md` | 0 |
| IMP-DX-DORA-010〜020 | DORA 4 keys 11 ID（Deployment Frequency / Lead Time / Change Failure Rate / MTTR の計測点、Severity 分離、postmortem 連動、Backstage Scorecards 連動） | `95_DXメトリクス/10_DORA_4keys/01_DORA_4keys計測.md` | リリース時点〜採用初期 |

予約: IMP-DX-DORA-021〜029 / IMP-DX-030〜099（リリース時点: time-to-first-commit 節 / SPACE フレームワーク追加）。

## IMP-TRACE（99 章 索引）

本章自身の原則 7 件のみ採番。実装 ID は運用に入ってから追加するため リリース時点 以降で採番。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-TRACE-POL-001〜007 | 12 接頭辞固定 / 予約帯 001-099 固定 / 1 判断 = 1 ID 原子性 / 本章を採番最終更新先 / ADR/DS-SW-COMP/NFR 双方向リンク / Backstage catalog 対応 リリース時点 確立 / 改訂履歴本章集約 | `99_索引/00_方針/01_索引運用原則.md` | 0 |

予約: IMP-TRACE-010〜099（リリース時点: 整合性 CI チェック節 / リリース時点: catalog-info.yaml スキーマ検証節）。

## 並列索引: IMP-DIR（00 章 ディレクトリ設計）

00 章の `IMP-DIR-*` は `00_ディレクトリ設計/90_トレーサビリティ/01_IMP-DIR_ID一覧.md` で自律管理される 145 件の予約枠（ROOT / T1 / T2 / T3 / INFRA / OPS / COMM / SPARSE の 8 サブ接頭辞）を持つ。リリース時点で 57 件採番済・88 件予約残。本章とは守備範囲が重複しないため再掲しないが、改訂影響範囲を追う際は必ず併読する（IMP-TRACE-POL-001 で並列管理として明示）。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- 並列索引: [`../../00_ディレクトリ設計/90_トレーサビリティ/01_IMP-DIR_ID一覧.md`](../../00_ディレクトリ設計/90_トレーサビリティ/01_IMP-DIR_ID一覧.md)
- ADR 対応: [`../10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- DS-SW-COMP 対応: [`../20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md`](../20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md)
- NFR 対応: [`../30_NFR対応表/01_NFR-IMP対応マトリクス.md`](../30_NFR対応表/01_NFR-IMP対応マトリクス.md)
- 改訂履歴: [`../90_改訂履歴/01_改訂履歴.md`](../90_改訂履歴/01_改訂履歴.md)
