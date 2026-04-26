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
| IMP-CODEGEN | 20 コード生成 | 7 | 32 | 39 | 60 | BUF（buf 生成）/ OAS（OpenAPI）/ SCF（Scaffold CLI）/ GLD（Golden snapshot） |
| IMP-CI | 30 CI/CD | 7 | 48 | 55 | 44 | RWF（reusable workflow）/ PF（path-filter）/ HAR（Harbor/Trivy）/ QG（quality gate）/ BP（branch protection） |
| IMP-DEP | 40 依存管理 | 7 | 26 | 33 | 66 | REN（Renovate 中央運用）/ SBM（SBOM 差分監視）/ LIC（ライセンス判定） |
| IMP-DEV | 50 開発者体験 | 7 | 42 | 49 | 50 | DC（Dev Container）/ GP（Golden Path）/ SO（Scaffold）/ BSN（Backstage）/ ONB（Onboarding） |
| IMP-OBS | 60 観測性 | 7 | 68 | 75 | 24 | OTEL / LGTM / PYR / SLO / SLI / EB / INC / RB |
| IMP-REL | 70 リリース | 7 | 47 | 54 | 45 | ARG（ArgoCD）/ PD（Progressive Delivery）/ FFD（flagd）/ AT（AnalysisTemplate）/ RB（Rollback Runbook） |
| IMP-SUP | 80 サプライチェーン | 7 | 46 | 53 | 46 | COS（cosign）/ SBM（CycloneDX SBOM）/ SLSA（Provenance v1）/ FOR（Forensics）/ FLG（flag 定義署名検証） |
| IMP-SEC | 85 Identity | 7 | 59 | 66 | 33 | KC（Keycloak）/ SP（SPIRE）/ OBO（OpenBao）/ CRT（cert-manager）/ REV（退職 revoke） |
| IMP-POL | 90 ガバナンス | 7 | 37 | 44 | 55 | KYV（Kyverno）/ ADR（ADR プロセス）/ TR（Technology Radar）/ STR（STRIDE） |
| IMP-DX | 95 DX メトリクス | 7 | 50 | 57 | 42 | DORA / SPC（SPACE）/ SCAF（Scaffold 利用率）/ TFC（time-to-first-commit）/ EMR（EM レポート） |
| IMP-TRACE | 99 索引 | 7 | 20 | 27 | 72 | CI（整合性 CI）/ CAT（catalog-info 検証） |

リリース時点 全体で採番済 ID は計 574 件（POL 84 件 + 実装 490 件）、予約残は 614 件。実装 490 件は リリース時点 段階の「核心節 19 ファイル + 依存管理 3 節 + ガバナンス 4 節 + コード生成 OAS / GLD 2 節 + CI/CD PF / QG / BP 3 節 + 開発者体験 SO / BSN / ONB 3 節 + 観測性 LGTM / PYR / EB / RB 4 節 + リリース FFD / AT / RB 3 節 + サプライチェーン SBM / SLSA / FLG 3 節 + Identity OBO / CRT 2 節 + DX SPC / SCAF / TFC / EMR 4 節 + 索引 整合性 CI / catalog-info 検証 2 節」から抽出した純粋な採番で、POL を除く。

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

Protobuf / OpenAPI からの生成と Scaffold CLI による雛形生成、および Golden snapshot による生成器挙動回帰検証の判断を採番する。方針は `20_コード生成設計/00_方針/01_コード生成原則.md`、実装は BUF（buf 生成パイプライン）/ OAS（OpenAPI 3 ジェネレータ）/ SCF（Scaffold CLI）/ GLD（Golden snapshot）の 4 節に分割される。BUF と OAS は同じ「契約 → 言語別生成 → DO NOT EDIT 強制 → breaking ゲート」のパターンを Protobuf / HTTP REST にそれぞれ適用したもの。GLD は CLI バージョン更新時の生成パターン変化を最小代表サンプルで pin して人間レビューに乗せるための仕組みで、drift 検出（BUF-014 / OAS-025）と補完関係にある。

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
| IMP-CODEGEN-OAS-020 | `src/contracts/openapi/v1/` 単一 yaml ディレクトリ + 3 ジェネレータ設定の物理分離 | `20_コード生成設計/20_OpenAPI/01_OpenAPI生成パイプライン.md` | 0 |
| IMP-CODEGEN-OAS-021 | portal / admin / external-webhook の 3 系統限定（OpenAPI 採用範囲の境界） | 同上 | 0 |
| IMP-CODEGEN-OAS-022 | openapi-typescript / oapi-codegen / NSwag.MSBuild の言語別採用 | 同上 | 0 |
| IMP-CODEGEN-OAS-023 | 生成先物理パス分離（tier3 web / tier3 BFF / tier1 external / tier2 / SDK） | 同上 | 0 |
| IMP-CODEGEN-OAS-024 | oasdiff `--fail-on ERR` の必須ゲート | 同上 | 0 |
| IMP-CODEGEN-OAS-025 | `tools/codegen/verify-openapi-drift.sh` による DO NOT EDIT 強制 | 同上 | 0 |
| IMP-CODEGEN-OAS-026 | `tools/codegen/openapi.versions` による 4 CLI バージョン固定 | 同上 | 0 |
| IMP-CODEGEN-OAS-027 | v1 → v2 ディレクトリ分岐による breaking 変更経路 | 同上 | 採用初期 |
| IMP-CODEGEN-SCF-030 | Rust 実装の Scaffold CLI（`src/platform/scaffold/`） | `20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md` | 0 |
| IMP-CODEGEN-SCF-031 | Backstage Software Template 互換の `template.yaml` 採用 | 同上 | 採用初期 |
| IMP-CODEGEN-SCF-032 | tier2 / tier3 テンプレート配置分離 | 同上 | 0 |
| IMP-CODEGEN-SCF-033 | `catalog-info.yaml` の自動生成と CODEOWNERS 連動 | 同上 | 0 |
| IMP-CODEGEN-SCF-034 | SRE + Security 二重承認の branch protection 強制 | 同上 | 0 |
| IMP-CODEGEN-SCF-035 | golden snapshot 検証と `UPDATE_GOLDEN=1` 承認プロセス | 同上 | 0 |
| IMP-CODEGEN-SCF-036 | テンプレート semver バージョニング | 同上 | 0 |
| IMP-CODEGEN-SCF-037 | リリース時点 の Backstage UI 統合経路（custom action 化） | 同上 | 採用初期 |
| IMP-CODEGEN-GLD-040 | `tests/codegen/golden-{input,output}/` の物理配置と本番 contracts からの分離 | `20_コード生成設計/40_Golden_snapshot/01_Golden_snapshot.md` | 0 |
| IMP-CODEGEN-GLD-041 | Protobuf / OpenAPI / Scaffold 3 系統の最小代表サンプル原則 | 同上 | 0 |
| IMP-CODEGEN-GLD-042 | `run-golden-snapshot.sh` の exit code 3 値設計（0 / 1 / 2）と CI ラベル誘導 | 同上 | 0 |
| IMP-CODEGEN-GLD-043 | `update-golden-snapshot.sh` の物理分離（誤上書き防止） | 同上 | 0 |
| IMP-CODEGEN-GLD-044 | snapshot 更新 PR の記載要件と CODEOWNERS 必須レビュー | 同上 | 0 |
| IMP-CODEGEN-GLD-045 | reusable workflow `codegen-golden-snapshot` の trigger 条件と月次 schedule | 同上 | 0 |
| IMP-CODEGEN-GLD-046 | `normalize.sh` による非決定要素の正規化ポリシー | 同上 | 0 |
| IMP-CODEGEN-GLD-047 | drift 検出と golden snapshot の役割分担文書化（POL-004 / POL-005 と CLI 固定 BUF-015 / OAS-026 の関係） | 同上 | 0 |

予約: IMP-CODEGEN-BUF-018〜019 / IMP-CODEGEN-OAS-028〜029 / IMP-CODEGEN-SCF-038〜039 / IMP-CODEGEN-GLD-048〜049 / IMP-CODEGEN-050〜099（リリース時点 の Backstage 統合 / 5 言語目追加用）。

## IMP-CI（30 章 CI/CD 設計）

7 段階 CI（lint → test → build → push → sign → deploy → verify）と Harbor/Trivy/cosign の統合判断を採番する。方針は `30_CI_CD設計/00_方針/01_CI_CD原則.md`、実装は RWF（reusable workflow）/ PF（path-filter 選択ビルド）/ HAR（Harbor/Trivy push）/ QG（quality gate）/ BP（branch protection）の 5 節。番号空間は本章全体で重複しない（接頭辞横断の単一空間）方針を採り、QG が章番号 30 でありながら ID レンジが 060 始まりなのは HAR が 040〜051 を消費して 040〜049 のレンジを超過したため。詳細採番ルールは [`30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md`](../../30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md) を参照。

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
| IMP-CI-PF-030 | `dorny/paths-filter@v3` の採用と `@v3` major 固定 | `30_CI_CD設計/20_path_filter選択ビルド/01_path_filter選択ビルド.md` | 0 |
| IMP-CI-PF-031 | `tools/ci/path-filter/filters.yaml` の単一真実源化（10 ビルド章と本章で共有） | 同上 | 0 |
| IMP-CI-PF-032 | 4 軸（tier / 言語 / workspace / contracts 横断）の判定構造 | 同上 | 0 |
| IMP-CI-PF-033 | `contracts=true → sdk-all=true` 強制昇格の物理化 | 同上 | 0 |
| IMP-CI-PF-034 | `_reusable-*.yml` への filter outputs 伝搬と起動条件 | 同上 | 0 |
| IMP-CI-PF-035 | `tools/ci/path-filter/run-golden-test.sh` による filter 変更保護 | 同上 | 0 |
| IMP-CI-PF-036 | 集約 job `ci-overall` 1 本のみを必須 status check とする運用 | 同上 | 0 |
| IMP-CI-PF-037 | キャッシュキーへの tier / 言語伝搬による衝突回避 | 同上 | 0 |
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
| IMP-CI-QG-060 | 4 ゲート（fmt / lint / unit-test / coverage）の順序固定 | `30_CI_CD設計/30_quality_gate/01_quality_gate.md` | 0 |
| IMP-CI-QG-061 | fmt は `--check` モード固定で自動修正禁止 | 同上 | 0 |
| IMP-CI-QG-062 | lint は warning 全て error 化（`-D warnings` / `--max-warnings 0`） | 同上 | 0 |
| IMP-CI-QG-063 | `tools/lint/` 配下の単一真実源化（tier 別独自ルール禁止） | 同上 | 0 |
| IMP-CI-QG-064 | unit-test は外部依存モック必須（integration-test は別 workflow） | 同上 | 0 |
| IMP-CI-QG-065 | カバレッジ段階導入（リリース時点 計測のみ → 採用初期 80% → 運用拡大 90%） | 同上 | 0 |
| IMP-CI-QG-066 | Cobertura XML 統一とベンダー SaaS 非送信 | 同上 | 0 |
| IMP-CI-QG-067 | 各ゲート failed 時の後段 skip 伝搬構造 | 同上 | 0 |
| IMP-CI-BP-070 | 必須 status check は `ci-overall` 1 本のみ（個別 job 不可） | `30_CI_CD設計/50_branch_protection/01_branch_protection.md` | 0 |
| IMP-CI-BP-071 | strict mode 有効化（main 最新 commit を含む状態でのみ merge） | 同上 | 0 |
| IMP-CI-BP-072 | 必須レビュー数（PoC 1 / 拡大期 2）と CODEOWNERS 自動指名 | 同上 | 採用初期 |
| IMP-CI-BP-073 | squash merge 強制 / linear history 必須 | 同上 | 0 |
| IMP-CI-BP-074 | 署名コミット必須（SSH / GPG / Web UI） | 同上 | 0 |
| IMP-CI-BP-075 | 管理者にも適用 / direct push 禁止 / merge queue は採用拡大期 | 同上 | 採用初期 |
| IMP-CI-BP-076 | terraform-provider-github による rule の Git 管理（IaC 化） | 同上 | 0 |
| IMP-CI-BP-077 | `release/*` ブランチに main と同一 rule を適用 | 同上 | 0 |

予約: IMP-CI-RWF-022〜029 / IMP-CI-PF-038〜039 / IMP-CI-HAR-052〜059 / IMP-CI-QG-068〜069 / IMP-CI-BP-078〜079 / IMP-CI-080〜099（リリース時点: ポリシー段 / merge queue 拡張）。

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

Paved Road 思想・10 役 Dev Container・Golden Path examples・Scaffold CLI 運用・Backstage 連携・Onboarding 動線の 6 本柱を採番する。方針は `50_開発者体験設計/00_方針/01_開発者体験原則.md`、実装は DC（Dev Container）/ GP（Golden Path）/ SO（Scaffold）/ BSN（Backstage）/ ONB（Onboarding）の 5 節。

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
| IMP-DEV-SO-030 | Scaffold 経由を全新規 component 作成の必須経路化 | `50_開発者体験設計/30_Scaffold_CLI運用/01_Scaffold_CLI運用.md` | 0 |
| IMP-DEV-SO-031 | CLI 直接 / Backstage UI 2 経路の同等性（共有 scaffold engine） | 同上 | 0 |
| IMP-DEV-SO-032 | 出力同等性の golden test バイト一致検証 | 同上 | 0 |
| IMP-DEV-SO-033 | 3 sub-command 固定（`list` / `new` / `new --dry-run`） | 同上 | 0 |
| IMP-DEV-SO-034 | 4 入力項目固定（template / name / owner / system） | 同上 | 0 |
| IMP-DEV-SO-035 | 4 出力 artifact 必須（コード雛形 / catalog-info.yaml / CODEOWNERS / docs README） | 同上 | 0 |
| IMP-DEV-SO-036 | 出力後 `make check` 通過判定とロールバック | 同上 | 0 |
| IMP-DEV-SO-037 | Backstage 自動 discovery 連動と template 更新影響範囲可視化 | 同上 | 採用初期 |
| IMP-DEV-BSN-040 | entity 5 種別固定（Component / API / Group / System）と種別追加 ADR 必須 | `50_開発者体験設計/40_Backstage連携/01_Backstage連携.md` | 0 |
| IMP-DEV-BSN-041 | Component path 直下同居 vs Group/System 集約配置の使い分け | 同上 | 0 |
| IMP-DEV-BSN-042 | catalog-info.yaml 必須 5 属性と `k1s0.io/template-version` annotation | 同上 | 0 |
| IMP-DEV-BSN-043 | GitHub provider 5 分 polling と webhook 不採用判断 | 同上 | 0 |
| IMP-DEV-BSN-044 | CI 段の `backstage-cli catalog:validate` 必須化 | 同上 | 0 |
| IMP-DEV-BSN-045 | `@backstage/cli` バージョン pin と Backstage 本体同期更新 | 同上 | 0 |
| IMP-DEV-BSN-046 | Catalog Errors の `@k1s0/platform-dx` 日次目視運用 | 同上 | 0（Slack 連動は採用初期） |
| IMP-DEV-BSN-047 | TechInsights 4 ファクト（coverage / lint / sbom / cosign）実装 | 同上 | 0 |
| IMP-DEV-BSN-048 | catalog-info.yaml を真実源とし Backstage は表示層とする復旧構造 | 同上 | 0 |
| IMP-DEV-ONB-050 | time-to-first-commit を SLI 化、Day 1 4 時間以内（採用拡大期 2 時間以内） | `50_開発者体験設計/50_オンボーディング/01_オンボーディング.md` | 0 |
| IMP-DEV-ONB-051 | Day 0 HR / IT / メンター責務分担と入社前々日 Backstage Group 登録 PR | 同上 | 0 |
| IMP-DEV-ONB-052 | Day 1 4 step（役割確定 / 環境構築 / Hello World / 微小 PR）と時間予算 | 同上 | 0 |
| IMP-DEV-ONB-053 | `goldenpath/<role>-hello.md` 5 step 完走絶対要件と崩壊時メンター修繕義務 | 同上 | 0 |
| IMP-DEV-ONB-054 | 微小 PR 儀式化設計と範囲（typo / catalog-info / docs） | 同上 | 0 |
| IMP-DEV-ONB-055 | SLI 計測経路（onboardingTimeFactRetriever / Scaffold 自動フッタ） | 同上 | 0（ゲート化は採用初期） |
| IMP-DEV-ONB-056 | Week 1 学習リスト（ADR-DIR-001/003 / ADR-TIER1-001 / 90_knowledge）と Scorecards 連動 | 同上 | 0 |
| IMP-DEV-ONB-057 | Week 2〜4 実 task 着手と複数 cone 併用段階導入 | 同上 | 0 |
| IMP-DEV-ONB-058 | Month 1 自走判定 4 軸（PR 量 / レビュー受領 / Slack / オンコール） | 同上 | 0 |
| IMP-DEV-ONB-059 | `onboarding-stumble` label による動線詰まり記録と月次帰着先 PR | 同上 | 0 |

予約: IMP-DEV-DC-018〜019 / IMP-DEV-GP-027〜029 / IMP-DEV-SO-038〜039 / IMP-DEV-BSN-049 / IMP-DEV-060〜099（採用拡大期: Scorecards ゲート化 / Webhook 連動 / 自動 onboarding bot 等）。

## IMP-OBS（60 章 観測性設計）

OTel Collector 配置・LGTM Stack 配置・Pyroscope 統合・SLO/SLI 定義・Error Budget 運用・Incident Taxonomy・Runbook 連携の 7 本柱を採番する。方針は `60_観測性設計/00_方針/01_観測性原則.md`、実装は OTEL（Collector）/ LGTM（Loki/Grafana/Tempo/Mimir）/ PYR（Pyroscope）/ SLO（SLI/SLO 定義）/ EB（Error Budget 状態機械）/ INC（Incident Taxonomy）/ RB（Runbook）の 7 節。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-OBS-POL-001〜007 | OTel Collector 集約 / LGTM AGPL 分離維持 / Google SRE Book 準拠 / Incident Taxonomy 可用性×セキュリティ統合 / Error Budget 100% 消費時 feature 凍結 / Runbook と SLI 紐付け / DORA 4 keys は 95 章 | `60_観測性設計/00_方針/01_観測性原則.md` | 0 |
| IMP-OBS-OTEL-010〜019 | Agent/Gateway 二層、tail sampling、PII transform、AGPL 境界など OTel Collector 10 ID | `60_観測性設計/10_OTel_Collector配置/01_OTel_Collector配置.md` | リリース時点〜採用初期 |
| IMP-OBS-LGTM-020 | LGTM 4 コンポーネントを `observability-lgtm` namespace 集約配置 | `60_観測性設計/20_LGTM_Stack/01_LGTM_Stack配置.md` | 0 |
| IMP-OBS-LGTM-021 | Loki/Mimir/Tempo を StatefulSet、Grafana を Deployment + Postgres state | 同上 | 0 |
| IMP-OBS-LGTM-022 | NetworkPolicy で ingress を OTel Gateway と Grafana のみ許可 | 同上 | 0 |
| IMP-OBS-LGTM-023 | S3 互換オブジェクトストレージ採用（リリース時点 MinIO） | 同上 | 0（採用初期で外部 S3） |
| IMP-OBS-LGTM-024 | 保持期間 Loki 30/365 日 / Mimir 30 日/13 ヶ月 / Tempo 14 日 | 同上 | 0 |
| IMP-OBS-LGTM-025 | Grafana 匿名閲覧禁止 + Keycloak OIDC SSO 必須 | 同上 | 0 |
| IMP-OBS-LGTM-026 | datasource を Grafana → 各 backend へ直接接続 | 同上 | 0 |
| IMP-OBS-LGTM-027 | Postgres 日次 backup + S3 バージョニング/replication 冗長化 | 同上 | 0 |
| IMP-OBS-LGTM-028 | Collector Gateway の disk queue 1 GB バッファ | 同上 | 0 |
| IMP-OBS-LGTM-029 | 復旧優先順位 Mimir → Loki → Tempo → Grafana 固定 | 同上 | 0 |
| IMP-OBS-PYR-030 | 4 ランタイム（Rust/Go/Node/.NET）SDK push 主、tags 必須注入 | `60_観測性設計/30_Pyroscope/01_Pyroscope統合.md` | 0 |
| IMP-OBS-PYR-031 | tier1 Rust の `otel-util` crate に Pyroscope 初期化集約 | 同上 | 0 |
| IMP-OBS-PYR-032 | Linux Node に Grafana Alloy + eBPF pull 補完 | 同上 | 0 |
| IMP-OBS-PYR-033 | Pyroscope は OTel Collector 外、OTLP profiles GA で再検討 | 同上 | 0 |
| IMP-OBS-PYR-034 | Tempo span attribute `pyroscope.profile.id` で双方向 link | 同上 | 0 |
| IMP-OBS-PYR-035 | Pyroscope server を `observability-lgtm` AGPL 同居配置 | 同上 | 0 |
| IMP-OBS-PYR-036 | 保持 14 日 hot / 30 日 cold、長期は nightly aggregate | 同上 | 0 |
| IMP-OBS-PYR-037 | Grafana datasource 3 ビュー（Flame Graph / Diff / Profile from Trace） | 同上 | 0 |
| IMP-OBS-PYR-038 | nightly diff で regression 自動検出 | 同上 | 採用初期 |
| IMP-OBS-PYR-039 | 障害時 SDK 5 分 buffer / Alloy 100 MB buffer | 同上 | 0 |
| IMP-OBS-SLO-040〜047 | tier1 公開 11 API の p99 SLO 階層、Runbook 一対一対応、外部 SLA 99% との乖離許容など | `60_観測性設計/40_SLO_SLI定義/01_tier1_公開11API_SLO_SLI.md` | 0 |
| IMP-OBS-SLI-003 | Availability SLI 初期定義（リリース時点 ブートストラップ）| `60_観測性設計/40_SLO_SLI定義/01_tier1_公開11API_SLO_SLI.md` | 0 |
| IMP-OBS-EB-050 | 28 日 rolling window で budget 計算、15 分平均で状態判定 | `60_観測性設計/50_ErrorBudget運用/01_ErrorBudget運用.md` | 0 |
| IMP-OBS-EB-051 | 4 状態（HEALTHY/WARNING/ALERT/FROZEN）と境界 50/25/0% | 同上 | 0 |
| IMP-OBS-EB-052 | FROZEN 時 Argo CD `selfHeal: false` 切替で sync 停止 | 同上 | 0 |
| IMP-OBS-EB-053 | budget 自動回復のみ、手動リセット禁止 | 同上 | 0 |
| IMP-OBS-EB-054 | セキュリティ hotfix（CVSS 9.0+）の `hotfix/sec-` bypass | 同上 | 0 |
| IMP-OBS-EB-055 | ダッシュボード 4 要素 + simplified dashboard 二段提供 | 同上 | 0 |
| IMP-OBS-EB-056 | Mimir 障害時は安全側 block 判定（pessimistic gate） | 同上 | 0 |
| IMP-OBS-EB-057 | FROZEN 到達は post-mortem 自動起票、半年 2 回で SLO 見直し ADR | 同上 | 0 |
| IMP-OBS-INC-060〜071 | Incident Taxonomy の AVL / SEC 統合、Sev1/Sev2 × Runbook 4 セル、PagerDuty 連動、NFR-E-SIR-002 72 時間通告など 12 ID | `60_観測性設計/60_Incident_Taxonomy/01_Incident_Taxonomy統合分類.md` | 0 |
| IMP-OBS-RB-080 | ID 体系 `<tier>.<service>.<sli_kind>.<symptom>` 1:1 結合 | `60_観測性設計/70_Runbook連携/01_Runbook連携.md` | 0 |
| IMP-OBS-RB-081 | リリース時点 Runbook 15 本（5 領域 × 3 本）配置 | 同上 | 0 |
| IMP-OBS-RB-082 | Runbook 5 セクション固定（症状/影響/一次/根本/検証） | 同上 | 0 |
| IMP-OBS-RB-083 | CI lint 3 種（1:1/5 セクション/メタデータ）の `ci-overall` 必須 | 同上 | 0 |
| IMP-OBS-RB-084 | Alertmanager rule の `annotations.runbook_url` 必須 | 同上 | 0 |
| IMP-OBS-RB-085 | Runbook 起動 3 経路（PagerDuty / Grafana / post-mortem）公式化 | 同上 | 0 |
| IMP-OBS-RB-086 | post-mortem 24h 以内の Runbook 改訂 PR 必須化 | 同上 | 0 |
| IMP-OBS-RB-087 | 段階拡大（リリース時点 15 → 採用初期 30 → 採用拡大期 50） | 同上 | 0 |
| IMP-OBS-RB-088 | 採用拡大期 50 本超で Runbook 索引 2 軸検索化 | 同上 | 採用拡大期 |
| IMP-OBS-RB-089 | GitHub 障害時の MinIO 同期 + ローカル `~/.k1s0-runbooks/` | 同上 | 採用初期 |

予約: IMP-OBS-SLO-048〜059 / IMP-OBS-INC-072〜079 / IMP-OBS-090〜099（採用拡大期: synthetic monitoring / 合成監視 / 顧客影響可視化）。

## IMP-REL（70 章 リリース設計）

ArgoCD による GitOps と Argo Rollouts による Progressive Delivery の判断を採番する。方針は `70_リリース設計/00_方針/01_リリース原則.md`、実装は ARG（ArgoCD App 構造）/ PD（Progressive Delivery / AnalysisTemplate）の 2 節。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-REL-POL-001〜007 | GitOps 一本化 / Progressive Delivery 必須 / Canary AnalysisTemplate 強制 / 手動 rollback 15 分以内 / Rollback 経路単一化 / flag 即時切替分離 / App-of-Apps 構造 | `70_リリース設計/00_方針/01_リリース原則.md` | 0 |
| IMP-REL-ARG-010〜017 | App-of-Apps、ApplicationSet、sync policy、環境別 overlays など ArgoCD 8 ID | `70_リリース設計/10_ArgoCD_App構造/01_ArgoCD_App構造.md` | リリース時点〜採用初期 |
| IMP-REL-PD-020〜028 | Canary 段階（5/25/50/100）、AnalysisTemplate、failureLimit、手動 rollback、CR 定義など Argo Rollouts 9 ID | `70_リリース設計/20_ArgoRollouts_PD/01_ArgoRollouts_PD設計.md` | リリース時点〜採用初期 |
| IMP-REL-FFD-030〜039 | flagd 4 種別フラグ / cosign keyless 署名 / Kyverno 検証 / sidecar 配置 / OpenFeature SDK 4 言語 / 評価ログ OTel 連動 10 ID | `70_リリース設計/30_flagd_フィーチャーフラグ/01_flagd_フィーチャーフラグ設計.md` | リリース時点〜採用初期 |
| IMP-REL-AT-040〜049 | AnalysisTemplate 共通 5 本（error-rate/latency/cpu/dependency/EB-burn）、固有テンプレ継承、Mimir provider、Scaffold 自動挿入、月次カバレッジ 10 ID | `70_リリース設計/40_AnalysisTemplate/01_AnalysisTemplate設計.md` | リリース時点 |
| IMP-REL-RB-050〜059 | 5 段階 15 分タイムライン、`ops/scripts/rollback.sh` 1 コマンド化、Branch Protection 4-eyes、第二/第三経路、四半期演習、Postmortem 自動化 10 ID | `70_リリース設計/50_rollback_runbook/01_rollback_runbook設計.md` | リリース時点〜採用初期 |

予約: IMP-REL-ARG-018〜019 / IMP-REL-PD-029 / IMP-REL-060〜099（リリース時点: 環境パイプライン拡張 / 多リージョン / Release Train、採用後の運用拡大時: マルチクラスタ）。

## IMP-SUP（80 章 サプライチェーン設計）

cosign keyless 署名・CycloneDX SBOM・SLSA Provenance v1・Forensics Runbook・flag 定義署名検証の判断を採番する。方針は `80_サプライチェーン設計/00_方針/01_サプライチェーン原則.md`、実装は COS（cosign）/ SBM（CycloneDX SBOM）/ SLSA（Provenance v1）/ FOR（Forensics）/ FLG（flag 定義署名検証）の 5 節。本章のサブ接頭辞詳細逆引きは [`../../80_サプライチェーン設計/90_対応IMP-SUP索引/01_対応IMP-SUP索引.md`](../../80_サプライチェーン設計/90_対応IMP-SUP索引/01_対応IMP-SUP索引.md) を参照。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-SUP-POL-001〜007 | SLSA L2 先行 → L3 到達 / cosign keyless 必須 / SBOM 全アーティファクト添付 / Rekor 記録永続 / Forensics Runbook 起点 image hash / Kyverno 署名検証 / AGPL 分離エビデンス | `80_サプライチェーン設計/00_方針/01_サプライチェーン原則.md` | リリース時点〜採用初期 |
| IMP-SUP-COS-010〜018 | cosign keyless 9 ID（OIDC 発行、Rekor 記録、検証経路、鍵なし運用の可監査性、攻撃面） | `80_サプライチェーン設計/10_cosign署名/01_cosign_keyless署名.md` | 0 |
| IMP-SUP-SBM-020〜029 | CycloneDX SBOM 10 ID（4 言語生成器固定、syft + cdx-merge、cosign attest 配布、Kyverno 必須化、cyclonedx-cli diff、osv-scanner+grype 2 重照合、AGPL 検出フロー、3 年保管 + WORM スナップショット） | `80_サプライチェーン設計/20_CycloneDX_SBOM/01_CycloneDX_SBOM設計.md` | リリース時点〜採用初期 |
| IMP-SUP-SLSA-030〜039 | SLSA Provenance v1 10 ID（リリース時点 L2 = hosted runner + slsa-github-generator + cosign attest + Kyverno verifyAttestations / catalog 表示 / 虚偽申告検知 / 採用後 L3 = Hermetic + Isolated + 4-eyes + Reproducible） | `80_サプライチェーン設計/30_SLSA_プロビナンス/01_SLSA_プロビナンス設計.md` | リリース時点〜採用後の運用拡大時 |
| IMP-SUP-FOR-040〜048 | Forensics Runbook 9 ID（image hash → 影響範囲、SBOM 突合、48 時間 SLA、Kyverno 迂回検知、PagerDuty エスカレ） | `80_サプライチェーン設計/40_Forensics_Runbook/01_image_hash逆引き_Forensics.md` | リリース時点〜採用初期 |
| IMP-SUP-FLG-050〜057 | flag 定義署名検証 8 ID（cosign sign-blob keyless / OCI Artifact + Rekor / Kyverno verify-blob admission / 四半期棚卸し 3 ステップ / 検証失敗時 Forensics Sev1/Sev2 振り分け） | `80_サプライチェーン設計/50_flag_定義署名検証/01_flag_定義署名検証.md` | リリース時点〜採用初期 |

予約: IMP-SUP-COS-019 / IMP-SUP-FOR-049 / IMP-SUP-FLG-058〜059 / IMP-SUP-060〜099（採用後の運用拡大時: in-toto Layout 拡張 / オフサイト Sigstore 移行 / SBOM Dependency-Track 統合）。

## IMP-SEC（85 章 Identity 設計）

Keycloak / SPIRE / OpenBao / cert-manager / 退職 revoke / Istio mTLS の 6 原則と実装を採番する。他章と異なり実装 ID が突出して多いのは、Identity が人間 ID とワークロード ID の 2 系統を抱えるため。方針は `85_Identity設計/00_方針/01_Identity原則.md`。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-SEC-POL-001〜007 | 人間 ID Keycloak 集約 / ワークロード ID SPIRE / 退職 revoke 60 分以内 / OpenBao Secret 集約 / cert-manager 証明書自動更新 / Istio Ambient mTLS / GameDay 継続検証 | `85_Identity設計/00_方針/01_Identity原則.md` | 0 |
| IMP-SEC-KC-010〜022 | Keycloak realm 13 ID（tenant 分離、JWT claims、admin event、login flow、MFA、2FA、7 年監査保存） | `85_Identity設計/10_Keycloak_realm/01_Keycloak_realm設計.md` | 0 |
| IMP-SEC-SP-020〜035 | SPIRE/SPIFFE 16 ID（HA 3 replica、DaemonSet、PSAT 認証、SVID 発行、Dapr 統合、Istio 統合、GameDay） | `85_Identity設計/20_SPIRE_SPIFFE/01_SPIRE_SPIFFE設計.md` | リリース時点〜採用初期 |
| IMP-SEC-OBO-040〜049 | OpenBao 10 ID（Raft Integrated Storage 3 node HA / Auto-unseal AWS KMS / KV-v2・PKI・Transit 3 secret engine / Kubernetes Auth Method / Audit Device 二段保管 Loki 90 日 + S3 7 年 WORM / root token Sev1） | `85_Identity設計/30_OpenBao/01_OpenBao設計.md` | リリース時点〜採用初期 |
| IMP-SEC-CRT-060〜069 | cert-manager 10 ID（3 ClusterIssuer Let's Encrypt / Vault PKI / SelfSigned / Certificate CR rotationPolicy: Always / 5 分 Reconciliation Loop / istio-csr 経由 SPIRE SVID 統合 1h ローテ / CertificateRequest 7 年 WORM / Prometheus Alert 4 段階エスカレーション） | `85_Identity設計/40_cert-manager/01_cert-manager設計.md` | リリース時点〜採用初期 |
| IMP-SEC-REV-050〜059 | 退職 revoke Runbook 10 ID（起点通知、60 分 SLA、7 年監査ログ、Object Lock、Service Account 最小権限） | `85_Identity設計/50_退職時revoke手順/01_退職時revoke手順.md` | 0 |

予約: IMP-SEC-KEY-001〜009（鍵管理サブ接頭辞の予約枠。採用初期で OpenBao KV-v2 / PKI / Transit の鍵ライフサイクル ID を採番予定）/ IMP-SEC-KC-023〜029 / IMP-SEC-SP-036〜039 / IMP-SEC-OBO-050〜054（OBO 余裕枠は REV-050〜059 と被るため 050〜054 のみ将来 OBO 拡張用に内部予約）/ IMP-SEC-CRT-070〜079 / IMP-SEC-070〜099（採用初期: Keycloak SCIM 連携 / OpenBao Transit 高度暗号 / cert-manager DNS01 multi provider）。

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

DORA Four Keys / SPACE / Scaffold 利用率 / time-to-first-commit / EM 月次レポートの計測設計を採番する。方針は `95_DXメトリクス/00_方針/01_DXメトリクス原則.md`、実装は DORA（Four Keys 計測）/ SPC（SPACE）/ SCAF（Scaffold 利用率）/ TFC（time-to-first-commit）/ EMR（EM レポート）の 5 節。本章のサブ接頭辞詳細逆引きは [`../../95_DXメトリクス/90_対応IMP-DX索引/01_対応IMP-DX索引.md`](../../95_DXメトリクス/90_対応IMP-DX索引/01_対応IMP-DX索引.md) を参照。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-DX-POL-001〜007 | DORA Four Keys を リリース時点 計測 / Severity 別分離 / Deploy Frequency 分母定義 / MTTR ユーザ影響終点 / time-to-first-commit 独自 SLI / Backstage Scorecards / 四半期レビュー | `95_DXメトリクス/00_方針/01_DXメトリクス原則.md` | 0 |
| IMP-DX-DORA-010〜020 | DORA 4 keys 11 ID（Deployment Frequency / Lead Time / Change Failure Rate / MTTR の計測点、Severity 分離、postmortem 連動、Backstage Scorecards 連動） | `95_DXメトリクス/10_DORA_4keys/01_DORA_4keys計測.md` | リリース時点〜採用初期 |
| IMP-DX-SPC-021〜029 | SPACE 5 軸 9 ID（Survey 配信 / Activity span / Communication ログ / Efficiency opt-in / PII transform / WORM Snapshot / Scorecards / 個人ランキング化禁止 / EM 評価対象軸明示） | `95_DXメトリクス/20_SPACE/01_SPACE設計.md` | リリース時点〜運用拡大期 |
| IMP-DX-SCAF-030〜039 | Scaffold 利用率 10 ID（CLI / Software Template OTel 連動 / Off-Path 検出 / Catalog 走査 / Paved Road 健全度式 / Scorecards 表示 / EM レポート配信 / Sev3 通知 / template_id 別分解 / PII transform） | `95_DXメトリクス/30_Scaffold利用率/01_Scaffold利用率計測.md` | リリース時点〜運用拡大期 |
| IMP-DX-TFC-040〜049 | time-to-first-commit 10 ID（Stage 0〜4 OTel span / onboardingTimeFactRetriever / new_joiner_hash / Mimir histogram / SLI 化 / onboarding-stumble 集計 / cohort 推移 / Stage 別劣化検出 / 採用拡大期 2h 目標 / EM 連携） | `95_DXメトリクス/40_time_to_first_commit/01_time_to_first_commit計測.md` | リリース時点〜運用拡大期 |
| IMP-DX-EMR-050〜059 | EM 月次レポート 10 ID（Generator job / 4 入力統合クエリ / 3 形式生成 / Slack 配信 / Confluence 配信 / Backstage DX-Report Entity / 機械的閾値違反検出 / onboarding-stumble 統合 / 統計的有意性判定 / hash 化済データ流入 CI 検証） | `95_DXメトリクス/50_EMレポート/01_EM月次レポート設計.md` | リリース時点（1 件）〜運用拡大期 |

予約: IMP-DX-DORA-021〜029 / IMP-DX-060〜099（採用後の運用拡大時: SPACE 5 軸の Survey Plugin 後継 / Scaffold 利用率の閾値運用拡張 / TFC 採用拡大期 2h 目標達成施策 / EM レポートの ML ベースアクション提案）。

## IMP-TRACE（99 章 索引）

索引運用原則 7 件に加え、リリース時点 で整合性 CI（IMP-TRACE-CI-010〜019）と catalog-info.yaml スキーマ検証（IMP-TRACE-CAT-020〜029）の 2 サブ接頭辞を採番済。索引と参照網のドリフトを CI 段で構造的に検出する仕組みを物理化する。

| ID | 概要 | 所属ファイル | 適用段階 |
|---|---|---|---|
| IMP-TRACE-POL-001〜007 | 12 接頭辞固定 / 予約帯 001-099 固定 / 1 判断 = 1 ID 原子性 / 本章を採番最終更新先 / ADR/DS-SW-COMP/NFR 双方向リンク / Backstage catalog 対応 リリース時点 確立 / 改訂履歴本章集約 | `99_索引/00_方針/01_索引運用原則.md` | 0 |
| IMP-TRACE-CI-010 | 5 検証スクリプトの責務分離と `tools/trace-check/` 物理配置 | `99_索引/50_整合性CI/01_整合性CI設計.md` | 0 |
| IMP-TRACE-CI-011 | `check-grand-total.sh` 台帳 grand total 検算（サマリ vs 詳細行集計） | 同上 | 0 |
| IMP-TRACE-CI-012 | `check-cross-ref.sh` 90_対応索引と台帳の相互整合 | 同上 | 0 |
| IMP-TRACE-CI-013 | `check-orphan.sh` ADR / DS-SW-COMP / NFR マトリクス孤立 ID 検出 | 同上 | 0 |
| IMP-TRACE-CI-014 | `check-duplicate.sh` + `check-reserve.sh` ID 重複と予約帯外採番検出 | 同上 | 0 |
| IMP-TRACE-CI-015 | pre-commit hook ローカル検証（高速 3 検証 / 2 秒以内） | 同上 | 0 |
| IMP-TRACE-CI-016 | GHA reusable workflow `trace-check` の `ci-overall` 必須化 | 同上 | 0 |
| IMP-TRACE-CI-017 | 検証失敗時の Markdown レポートと 3 PR 連続失敗 Issue 起票 | 同上 | 採用初期 |
| IMP-TRACE-CI-018 | 月次 cron による `--strict` モード再検証（30 日以上残存孤立 ID 通知） | 同上 | 採用初期 |
| IMP-TRACE-CI-019 | 検証スクリプトの依存最小化（Bash + ripgrep + jq + yq）と nightly 依存検証 | 同上 | 0 |
| IMP-TRACE-CAT-020 | catalog-info.yaml 必須属性スキーマ（`tools/catalog-check/schema/catalog-info.schema.json`） | `99_索引/60_catalog-info検証/01_catalog-info検証設計.md` | 0 |
| IMP-TRACE-CAT-021 | `k1s0.io/template-version` annotation 必須化と SemVer 形式強制 | 同上 | 0 |
| IMP-TRACE-CAT-022 | `spec.lifecycle` 許可リスト（experimental / production / deprecated）強制 | 同上 | 0 |
| IMP-TRACE-CAT-023 | `spec.owner` Group 実在検証（Backstage Group catalog snapshot 参照） | 同上 | 0 |
| IMP-TRACE-CAT-024 | `spec.system` System 実在検証 | 同上 | 0 |
| IMP-TRACE-CAT-025 | Scaffold CLI dry-run との bit-by-bit 一致検証（編集可能フィールド除く） | 同上 | 採用初期 |
| IMP-TRACE-CAT-026 | GitHub repo 走査による Off-Path 検出（IMP-DX-SCAF-033 と同一バイナリ） | 同上 | 0 |
| IMP-TRACE-CAT-027 | pre-commit hook（軽 3 検証）+ GHA reusable workflow（全 5 検証） | 同上 | 0 |
| IMP-TRACE-CAT-028 | GHA workflow の `ci-overall` 必須化と path-filter 制御 | 同上 | 0 |
| IMP-TRACE-CAT-029 | 月次 cron による Off-Path Sev3 通知と EM 月次レポート連動 | 同上 | 採用初期 |

予約: IMP-TRACE-CI-NNN（020-029 / 030-039 など以降の検証ロジック追加用）/ IMP-TRACE-CAT-030〜039 / IMP-TRACE-040〜099（採用拡大期: lifecycle 自動遷移 / Scorecards テンプレ等）。

## 並列索引: IMP-DIR（00 章 ディレクトリ設計）

00 章の `IMP-DIR-*` は `00_ディレクトリ設計/90_トレーサビリティ/01_IMP-DIR_ID一覧.md` で自律管理される 145 件の予約枠（ROOT / T1 / T2 / T3 / INFRA / OPS / COMM / SPARSE の 8 サブ接頭辞）を持つ。リリース時点で 57 件採番済・88 件予約残。本章とは守備範囲が重複しないため再掲しないが、改訂影響範囲を追う際は必ず併読する（IMP-TRACE-POL-001 で並列管理として明示）。

## 関連ファイル

- 本章の原則: [`../00_方針/01_索引運用原則.md`](../00_方針/01_索引運用原則.md)
- 並列索引: [`../../00_ディレクトリ設計/90_トレーサビリティ/01_IMP-DIR_ID一覧.md`](../../00_ディレクトリ設計/90_トレーサビリティ/01_IMP-DIR_ID一覧.md)
- ADR 対応: [`../10_ADR対応表/01_ADR-IMP対応マトリクス.md`](../10_ADR対応表/01_ADR-IMP対応マトリクス.md)
- DS-SW-COMP 対応: [`../20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md`](../20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md)
- NFR 対応: [`../30_NFR対応表/01_NFR-IMP対応マトリクス.md`](../30_NFR対応表/01_NFR-IMP対応マトリクス.md)
- 改訂履歴: [`../90_改訂履歴/01_改訂履歴.md`](../90_改訂履歴/01_改訂履歴.md)
