# 01. IMP-DIR ID 一覧

本ファイルは `IMP-DIR-*` 設計 ID の全件を索引化する。各 ID に対応する文書ファイルと概要を列挙する。

## IMP-DIR ID 体系

ID 形式は `IMP-DIR-<サブ分類>-<3 桁通番>`。「ID 接頭辞（完全形）」列はサブ分類込みの実際に文書中に出現するプレフィクスを示し、grep などで全件抽出する際の検索キーとして使う。

| ID 接頭辞（完全形） | サブ分類 | 通番範囲 | 用途 |
|---|---|---|---|
| IMP-DIR-ROOT-\* | ROOT | 001-020 | ルートレイアウト |
| IMP-DIR-T1-\* | T1 | 021-040 | tier1 レイアウト |
| IMP-DIR-T2-\* | T2 | 041-055 | tier2 レイアウト |
| IMP-DIR-T3-\* | T3 | 056-070 | tier3 レイアウト |
| IMP-DIR-INFRA-\* | INFRA | 071-090 | infra レイアウト |
| IMP-DIR-OPS-\* | OPS | 091-110 | deploy + ops レイアウト |
| IMP-DIR-COMM-\* | COMM | 111-125 | 共通資産（tools/tests/examples/third_party） |
| IMP-DIR-SPARSE-\* | SPARSE | 126-145 | スパースチェックアウト運用 |

合計 145 件の予約枠。リリース時点での採番は約 40 件、残りは 運用蓄積後で追加採番する。

## ROOT 領域（001-020）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-ROOT-001 | `00_設計方針/01_ディレクトリ設計原則.md` | 責務境界原則 |
| IMP-DIR-ROOT-002 | `00_設計方針/01_ディレクトリ設計原則.md` | 依存方向原則 |
| IMP-DIR-ROOT-003 | `00_設計方針/01_ディレクトリ設計原則.md` | 5 階層深さ原則 |
| IMP-DIR-ROOT-004 | `00_設計方針/01_ディレクトリ設計原則.md` | README 必須原則 |
| IMP-DIR-ROOT-005 | `00_設計方針/01_ディレクトリ設計原則.md` | 生成物分離原則 |
| IMP-DIR-ROOT-006 | `00_設計方針/01_ディレクトリ設計原則.md` | 命名衝突回避原則 |
| IMP-DIR-ROOT-007 | `00_設計方針/01_ディレクトリ設計原則.md` | cone 統合原則 |
| IMP-DIR-ROOT-008 | `10_ルートレイアウト/01_ルート直下ファイル.md` | ルート直下ファイル配置 |
| IMP-DIR-ROOT-009 | `10_ルートレイアウト/02_src配下の層別分割.md` | src/ 層別分割 |
| IMP-DIR-ROOT-010 | `10_ルートレイアウト/03_横断ディレクトリ.md` | 横断ディレクトリ配置 |
| IMP-DIR-ROOT-011 | `10_ルートレイアウト/04_設定ファイル配置規約.md` | 設定ファイル配置規約 |
| IMP-DIR-ROOT-012 | `10_ルートレイアウト/05_依存方向ルール.md` | 依存方向ルール |
| IMP-DIR-ROOT-013 〜 020 | 運用蓄積後で採番 | 予約 |

## T1 領域（021-040）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-T1-021 | `20_tier1レイアウト/01_tier1全体配置.md` | tier1 全体配置 |
| IMP-DIR-T1-022 | `20_tier1レイアウト/02_contracts配置.md` | contracts 配置 |
| IMP-DIR-T1-023 | `20_tier1レイアウト/03_go_module配置.md` | Go module 配置 |
| IMP-DIR-T1-024 | `20_tier1レイアウト/04_rust_workspace配置.md` | Rust workspace 配置 |
| IMP-DIR-T1-025 | `20_tier1レイアウト/05_SDK配置.md` | SDK 配置 |
| IMP-DIR-T1-026 | `20_tier1レイアウト/06_生成コードの扱い.md` | 生成コードの扱い |
| IMP-DIR-T1-027 〜 040 | 運用蓄積後で採番 | 予約 |

## T2 領域（041-055）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-T2-041 | `30_tier2レイアウト/01_tier2全体配置.md` | tier2 全体配置 |
| IMP-DIR-T2-042 | `30_tier2レイアウト/02_dotnet_solution配置.md` | .NET solution 配置 |
| IMP-DIR-T2-043 | `30_tier2レイアウト/03_go_services配置.md` | Go services 配置 |
| IMP-DIR-T2-044 | `30_tier2レイアウト/04_サービス単位の内部構造.md` | サービス単位の内部構造 |
| IMP-DIR-T2-045 | `30_tier2レイアウト/05_テンプレート配置.md` | テンプレート配置 |
| IMP-DIR-T2-046 | `30_tier2レイアウト/06_依存管理.md` | 依存管理 |
| IMP-DIR-T2-047 〜 055 | 運用蓄積後で採番 | 予約 |

## T3 領域（056-070）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-T3-056 | `40_tier3レイアウト/01_tier3全体配置.md` | tier3 全体配置 |
| IMP-DIR-T3-057 | `40_tier3レイアウト/02_web_pnpm_workspace配置.md` | Web pnpm workspace 配置 |
| IMP-DIR-T3-058 | `40_tier3レイアウト/03_maui_native配置.md` | MAUI Native 配置 |
| IMP-DIR-T3-059 | `40_tier3レイアウト/04_bff配置.md` | BFF 配置 |
| IMP-DIR-T3-060 | `40_tier3レイアウト/05_レガシーラップ配置.md` | レガシーラップ配置 |
| IMP-DIR-T3-061 〜 070 | 運用蓄積後で採番 | 予約 |

## INFRA 領域（071-090）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-INFRA-071 | `50_infraレイアウト/01_infra全体配置.md` | infra 全体配置 |
| IMP-DIR-INFRA-072 | `50_infraレイアウト/02_k8sブートストラップ.md` | k8s ブートストラップ |
| IMP-DIR-INFRA-073 | `50_infraレイアウト/03_サービスメッシュ配置.md` | サービスメッシュ配置 |
| IMP-DIR-INFRA-074 | `50_infraレイアウト/04_Dapr_Component配置.md` | Dapr Component 配置 |
| IMP-DIR-INFRA-075 | `50_infraレイアウト/05_データ層配置.md` | データ層配置 |
| IMP-DIR-INFRA-076 | `50_infraレイアウト/06_セキュリティ層配置.md` | セキュリティ層配置 |
| IMP-DIR-INFRA-077 | `50_infraレイアウト/07_観測性配置.md` | 観測性配置 |
| IMP-DIR-INFRA-078 | `50_infraレイアウト/08_環境別パッチ配置.md` | 環境別パッチ配置 |
| IMP-DIR-INFRA-079 〜 082 | 運用蓄積後で採番 | 予約: feature-management / scaling / multi-region / backup-restore |
| IMP-DIR-INFRA-083 〜 086 | 運用蓄積後で採番 | 予約: hardware-profile / cost-optimization / green-ops / air-gap |
| IMP-DIR-INFRA-087 〜 090 | 採用後の運用拡大時で採番 | 予約: 将来のインフラ領域追加用（未割当バッファ） |

## OPS 領域（091-110）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-OPS-091 | `60_operationレイアウト/01_deploy配置_GitOps.md` | deploy 配置（GitOps） |
| IMP-DIR-OPS-092 | `60_operationレイアウト/02_ArgoCD_ApplicationSet配置.md` | ArgoCD ApplicationSet 配置 |
| IMP-DIR-OPS-093 | `60_operationレイアウト/03_Helm_charts配置.md` | Helm charts 配置 |
| IMP-DIR-OPS-094 | `60_operationレイアウト/04_Kustomize_overlays配置.md` | Kustomize overlays 配置 |
| IMP-DIR-OPS-095 | `60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md` | ops 配置 Runbook/Chaos/DR |
| IMP-DIR-OPS-096 | `60_operationレイアウト/06_Backstage_プラグイン配置.md` | Backstage プラグイン配置 |
| IMP-DIR-OPS-097 | `60_operationレイアウト/07_OpenTofu配置.md` | OpenTofu 配置 |
| IMP-DIR-OPS-098 〜 110 | 運用蓄積後で採番 | 予約 |

## COMM 領域（111-125）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-COMM-111 | `70_共通資産/01_tools配置.md` | tools 配置 |
| IMP-DIR-COMM-112 | `70_共通資産/02_tests配置.md` | tests 配置 |
| IMP-DIR-COMM-113 | `70_共通資産/03_examples配置.md` | examples 配置 |
| IMP-DIR-COMM-114 | `70_共通資産/04_third_party配置.md` | third_party 配置 |
| IMP-DIR-COMM-115 | `70_共通資産/05_devcontainer配置.md` | devcontainer 配置 |
| IMP-DIR-COMM-116 | `70_共通資産/06_codegen配置.md` | codegen 配置 |
| IMP-DIR-COMM-117 〜 125 | 運用蓄積後で採番 | 予約 |

## SPARSE 領域（126-145）

| ID | 文書 | 概要 |
|---|---|---|
| IMP-DIR-SPARSE-126 | `80_スパースチェックアウト運用/01_cone_mode設計原則.md` | cone mode 設計原則 |
| IMP-DIR-SPARSE-127 | `80_スパースチェックアウト運用/02_役割別cone定義.md` | 役割別 cone 定義 |
| IMP-DIR-SPARSE-128 | `80_スパースチェックアウト運用/03_初期クローンとオンボーディング.md` | 初期クローンとオンボーディング |
| IMP-DIR-SPARSE-129 | `80_スパースチェックアウト運用/04_役割切替運用.md` | 役割切替運用 |
| IMP-DIR-SPARSE-130 | `80_スパースチェックアウト運用/05_CI戦略とpath_filter統合.md` | CI 戦略と path-filter 統合 |
| IMP-DIR-SPARSE-131 | `80_スパースチェックアウト運用/06_注意点と既知問題.md` | 注意点と既知問題 |
| IMP-DIR-SPARSE-132 | `80_スパースチェックアウト運用/07_partial_clone_sparse_index.md` | partial clone / sparse index |
| IMP-DIR-SPARSE-133 〜 145 | 運用蓄積後で採番 | 予約 |

## 採番集計

| サブ | 範囲 | 採番済 | 予約残 | 合計 |
|---|---|---|---|---|
| ROOT | 001-020（20 件） | 12 | 8 | 20 |
| T1 | 021-040（20 件） | 6 | 14 | 20 |
| T2 | 041-055（15 件） | 6 | 9 | 15 |
| T3 | 056-070（15 件） | 5 | 10 | 15 |
| INFRA | 071-090（20 件） | 8 | 12 | 20 |
| OPS | 091-110（20 件） | 7 | 13 | 20 |
| COMM | 111-125（15 件） | 6 | 9 | 15 |
| SPARSE | 126-145（20 件） | 7 | 13 | 20 |
| **合計** | **001-145（145 件）** | **57** | **88** | **145** |
