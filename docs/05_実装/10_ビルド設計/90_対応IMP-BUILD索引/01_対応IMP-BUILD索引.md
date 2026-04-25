# 01. 対応 IMP-BUILD 索引

本ファイルは [`05_実装/10_ビルド設計/`](../README.md) 章配下で採番された全 `IMP-BUILD-*` ID を 1 ページに集約する横断索引である。各 ID から所在ファイル・対応原則・関連 ADR / DS / NFR への逆引きが可能で、PR レビュー時の影響範囲確認や、新規 ID 採番時の重複チェックを最短動線で行うために用意する。本索引は `IMP-BUILD-*` の正典とし、各章本文と齟齬が出た場合は本索引を改訂後に各章を更新する運用とする。

## 採番ルール

`IMP-BUILD-*` ID は次の規約で採番する。

- 形式: `IMP-BUILD-<接頭辞>-<番号>`
  - 接頭辞は本章配下のサブディレクトリ単位で割り当てる（例: Rust = `CW` = Cargo Workspace）
  - 番号は接頭辞内で 3 桁ゼロ埋めの連番（`010`, `011`, ...）
- 接頭辞別の番号レンジは 10 単位で予約し、欠番が出ても再利用しない
- 採番時は本索引の対応表を**先に**更新し、その後で本文ファイルに ID を埋め込む
- ID の意味（説明文）は本索引と本文ファイルで完全一致させる

接頭辞と章の対応は以下とする。

| 接頭辞 | 略称 | 所在 | 番号レンジ |
|---|---|---|---|
| `POL` | Policy | [`00_方針/01_ビルド設計原則.md`](../00_方針/01_ビルド設計原則.md) | 001 〜 009 |
| `CW` | Cargo Workspace | [`10_Rust_Cargo_workspace/01_Rust_Cargo_workspace.md`](../10_Rust_Cargo_workspace/01_Rust_Cargo_workspace.md) | 010 〜 019 |
| `GM` | Go Module | [`20_Go_module分離戦略/01_Go_module分離戦略.md`](../20_Go_module分離戦略/01_Go_module分離戦略.md) | 020 〜 029 |
| `TP` | TypeScript Pnpm | [`30_TypeScript_pnpm_workspace/01_TypeScript_pnpm_workspace.md`](../30_TypeScript_pnpm_workspace/01_TypeScript_pnpm_workspace.md) | 030 〜 039 |
| `DS` | Dotnet Sln | [`40_dotnet_sln境界/01_dotnet_sln境界.md`](../40_dotnet_sln境界/01_dotnet_sln境界.md) | 040 〜 049 |
| `PF` | Path Filter | [`50_選択ビルド判定/01_選択ビルド判定.md`](../50_選択ビルド判定/01_選択ビルド判定.md) | 050 〜 059 |
| `CS` | Cache Strategy | [`60_キャッシュ戦略/01_キャッシュ戦略.md`](../60_キャッシュ戦略/01_キャッシュ戦略.md) | 060 〜 069 |

## 全 ID 一覧（接頭辞別）

### POL: 設計原則（7 件）

| ID | 概要 |
|---|---|
| `IMP-BUILD-POL-001` | 言語ネイティブビルド優先（Bazel 不採用） |
| `IMP-BUILD-POL-002` | ワークスペース境界 = tier 境界 |
| `IMP-BUILD-POL-003` | 依存方向逆流の lint 拒否 |
| `IMP-BUILD-POL-004` | path-filter による選択ビルド |
| `IMP-BUILD-POL-005` | 3 層キャッシュ階層 |
| `IMP-BUILD-POL-006` | ビルド時間 SLI 計測 |
| `IMP-BUILD-POL-007` | 生成物 commit と隔離 |

### CW: Rust Cargo workspace（8 件）

| ID | 概要 |
|---|---|
| `IMP-BUILD-CW-010` | tier1 Rust workspace と SDK Rust workspace の 2 分割 |
| `IMP-BUILD-CW-011` | `[workspace.dependencies]` 集約方針（tier1 集約 / SDK 最小） |
| `IMP-BUILD-CW-012` | `rust-toolchain.toml` による patch レベル pin |
| `IMP-BUILD-CW-013` | sccache 連携と `incremental=false` |
| `IMP-BUILD-CW-014` | cargo-deny による license / ban / advisory 強制 |
| `IMP-BUILD-CW-015` | clippy lint と独自 deny lint（unwrap / panic 禁止） |
| `IMP-BUILD-CW-016` | cargo nextest（並列 / 60s タイムアウト / 80% coverage） |
| `IMP-BUILD-CW-017` | platform Cargo workspace 分離 |

### GM: Go module 分離戦略（8 件）

| ID | 概要 |
|---|---|
| `IMP-BUILD-GM-020` | tier1 Go / tier2 Go / SDK Go / BFF / tests の 5 module 分離 |
| `IMP-BUILD-GM-021` | replace ディレクティブによる内部 module 参照（3 経路のみ許容） |
| `IMP-BUILD-GM-022` | 独自 linter（`tools/ci/go-layer-check/`）による依存方向逆流検出 |
| `IMP-BUILD-GM-023` | `go.sum` 整合と `go mod tidy -diff` 検証 |
| `IMP-BUILD-GM-024` | path-filter 連動（tier 単位 module 検知） |
| `IMP-BUILD-GM-025` | `$GOCACHE` のリモート化（GitHub Actions cache、`go.sum` キー） |
| `IMP-BUILD-GM-026` | gopls / IDE 設定（`.vscode/settings.json`、`go.work` 不採用） |
| `IMP-BUILD-GM-027` | tests module の独立 go.mod による e2e 隔離 |

### TP: TypeScript pnpm workspace（8 件）

| ID | 概要 |
|---|---|
| `IMP-BUILD-TP-030` | tier3 web の単一 pnpm workspace と SDK TS の独立 package という 2 ビルド単位分離 |
| `IMP-BUILD-TP-031` | `apps/*` / `packages/*` の glob 分離による「リリース対象 / 内部依存先」識別 |
| `IMP-BUILD-TP-032` | `workspace:*` プロトコルによる内部参照、SDK は npm semver 経由参照の強制 |
| `IMP-BUILD-TP-033` | SDK の `files` フィールド限定による生成物・ソース非配布 |
| `IMP-BUILD-TP-034` | `pnpm --filter` を選択ビルドの統一表記とし、契約変更時は全 workspace 再ビルドを強制 |
| `IMP-BUILD-TP-035` | `eslint-plugin-boundaries` による tier 跨ぎ import の lint 段拒否 |
| `IMP-BUILD-TP-036` | pnpm store / GitHub Actions cache / Turbo Remote Cache の 3 層独立稼働 |
| `IMP-BUILD-TP-037` | `**/gen/**` のフォーマッタ・lint 除外による生成物ドリフト防止 |

### DS: Dotnet sln 境界（8 件）

| ID | 概要 |
|---|---|
| `IMP-BUILD-DS-040` | Tier2 / Native / Sdk の 3 sln 分割と sln 跨ぎ csproj 参照の禁止 |
| `IMP-BUILD-DS-041` | ProjectReference（同一 sln 内）と PackageReference（sln 跨ぎ）の使い分け強制 |
| `IMP-BUILD-DS-042` | `Directory.Build.props` による sln 横断制約（Nullable / 警告昇格 / sln 跨ぎ MSBuild ターゲット拒否） |
| `IMP-BUILD-DS-043` | `Directory.Packages.props` による CPM（Central Package Management）強制 |
| `IMP-BUILD-DS-044` | 各 csproj 直下 `Generated/` への生成物隔離と `generated_code = true` 宣言 |
| `IMP-BUILD-DS-045` | `packages.lock.json` の必須化（`RestorePackagesWithLockFile` + `--locked-mode`） |
| `IMP-BUILD-DS-046` | NuGet HTTP cache + GitHub Actions cache の 2 層運用、第 3 層は リリース時点 ADR 起票 |
| `IMP-BUILD-DS-047` | Sdk の `dotnet pack` による NuGet 配布形式と `<IsPackable>` 制御 |

### PF: Path filter（8 件）

| ID | 概要 |
|---|---|
| `IMP-BUILD-PF-050` | 4 段 path-filter による選択ビルド判定の全体構造 |
| `IMP-BUILD-PF-051` | 第 1 段 tier 判定（`src/<tier>/**` パターンマッチ） |
| `IMP-BUILD-PF-052` | 第 2 段 言語判定（拡張子 + lockfile による言語タグ） |
| `IMP-BUILD-PF-053` | 第 3 段 workspace 判定（各言語のビルド機構単位への射影） |
| `IMP-BUILD-PF-054` | 第 4 段 契約変更横断（SDK 4 言語 + tier1 全ビルド強制） |
| `IMP-BUILD-PF-055` | `dorny/paths-filter` + `tools/ci/path-filter/` Go スクリプトによる reusable workflow 化 |
| `IMP-BUILD-PF-056` | フェイルセーフ条件（`.github/workflows/**` / path-filter 自身の変更）の全強制ビルド |
| `IMP-BUILD-PF-057` | `docs` / `infra` / `tools` 単独変更時の軽量ジョブのみ起動 |

### CS: Cache strategy（8 件）

| ID | 概要 |
|---|---|
| `IMP-BUILD-CS-060` | 3 層独立稼働の全体設計（ローカル / CI / リモート） |
| `IMP-BUILD-CS-061` | 第 1 層ローカルキャッシュの言語別実体と容量管理 |
| `IMP-BUILD-CS-062` | 第 2 層 CI キャッシュ（GitHub Actions cache）のキー設計統一 |
| `IMP-BUILD-CS-063` | 第 3 層リモートキャッシュ（sccache / Turbo Remote Cache）の段階導入と判断基準 |
| `IMP-BUILD-CS-064` | キャッシュキーの基底（lockfile ハッシュ + OS + コンパイラ ver + env） |
| `IMP-BUILD-CS-065` | キャッシュヒット率の SLI 計測と `95_DXメトリクス/` 公開 |
| `IMP-BUILD-CS-066` | 各層失効時の影響範囲と自動復旧経路 |
| `IMP-BUILD-CS-067` | 第 3 層リモートキャッシュのアクセス制御（書き込みは OIDC 限定、サプライチェーン整合） |

## 採番済み件数まとめ

| 接頭辞 | 件数 | 残レンジ | 次番 |
|---|---|---|---|
| `POL` | 7 | 002 件（008-009） | `IMP-BUILD-POL-008` |
| `CW` | 8 | 002 件（018-019） | `IMP-BUILD-CW-018` |
| `GM` | 8 | 002 件（028-029） | `IMP-BUILD-GM-028` |
| `TP` | 8 | 002 件（038-039） | `IMP-BUILD-TP-038` |
| `DS` | 8 | 002 件（048-049） | `IMP-BUILD-DS-048` |
| `PF` | 8 | 002 件（058-059） | `IMP-BUILD-PF-058` |
| `CS` | 8 | 002 件（068-069） | `IMP-BUILD-CS-068` |
| **合計** | **55** | **14** | — |

各接頭辞のレンジが残り 2 件まで埋まっており、追加採番が見込まれる場合はレンジ拡張（例: `CW-010` 〜 `029` への 2 倍化）を ADR 起票で行う。レンジを跨ぐ単純連番は採番ミスの温床となるため、レンジ拡張は明示的な意思決定として記録する。

## 対応 ADR 逆引き

`IMP-BUILD-*` から参照される ADR を逆引きで一覧化する。各 ADR がどの IMP-BUILD ID に影響するかを把握する時に使う。

| ADR | 影響する IMP-BUILD ID |
|---|---|
| [ADR-TIER1-001](../../../02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md)（Go + Rust ハイブリッド） | POL-001, POL-002, CW-010, GM-020 |
| [ADR-TIER1-002](../../../02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md)（Protobuf gRPC 契約） | POL-007, CW-013, GM-024, TP-030〜032, DS-040〜041 |
| [ADR-TIER1-003](../../../02_構想設計/adr/ADR-TIER1-003-language-opacity.md)（内部言語不可視） | POL-002, POL-003, CW-010, GM-020, TP-032, DS-040 |
| [ADR-DIR-001](../../../02_構想設計/adr/ADR-DIR-001-contracts-elevation.md)（contracts 昇格） | POL-007, PF-054, TP-034, DS-044 |
| [ADR-DIR-003](../../../02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)（sparse cone） | PF-051 |
| [ADR-CICD-001](../../../02_構想設計/adr/ADR-CICD-001-argocd.md)（CI 構成） | PF-050〜057, CS-060〜067 |
| [ADR-SUP-001](../../../02_構想設計/adr/ADR-SUP-001-slsa-staged-adoption.md)（SLSA 段階導入） | CS-067 |

## 対応 DS-SW-COMP 逆引き

| DS-SW-COMP | 影響する IMP-BUILD ID |
|---|---|
| DS-SW-COMP-003（Dapr / Rust 二分） | POL-001, POL-002 |
| DS-SW-COMP-120（tier1 Go） | POL-002, GM-020 |
| DS-SW-COMP-121（tier1 Rust） | POL-002, CW-010 |
| DS-SW-COMP-122（contracts → 4 言語生成） | POL-007, PF-054, TP-030〜032, DS-040, CS-064 |
| DS-SW-COMP-129 / 130（SDK 配置と利用境界） | POL-002, CW-010, GM-020, TP-030, DS-040 |

## 対応 NFR 逆引き

| NFR | 影響する IMP-BUILD ID |
|---|---|
| [NFR-B-PERF-001](../../../03_要件定義/30_非機能要件/B_性能拡張性.md)（性能基盤 / p99 < 500ms） | POL-006, CS-060〜067 |
| [NFR-C-NOP-004](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（ビルド所要時間運用） | POL-004, POL-005, POL-006, PF-050〜057, CS-060〜067 |
| [NFR-C-MGMT-001](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（設定 Git 管理） | POL-007, 全 ID（lockfile / props / yml の commit 必須） |

## 上位索引との連携

本索引は [`05_実装/10_ビルド設計/`](../README.md) 章内の局所索引である。`IMP-BUILD-*` 全件を含むより上位の索引は以下に置かれる予定で、本索引はそこへ集約される位置付けとなる。

- [`05_実装/99_索引/`](../../99_索引/) — `IMP-*` 全接頭辞（DIR / BUILD / CODEGEN / CI / DEP / DEV / OBS / REL / SUP / SEC / POL / DX / TRACE）の横断索引
- [`04_概要設計/80_トレーサビリティ/01_設計ID索引.md`](../../../04_概要設計/80_トレーサビリティ/01_設計ID索引.md) — 設計 ID と実装 ID の突合せ

新規章追加・新規接頭辞追加・新規 ID 採番のたびに本索引と上位索引の両方を同期更新する。同期忘れは PR レビュー（[`docs-review-checklist`](../../../00_format/review_checklist.md)）の必須チェック項目とする。

## 関連章 / 参照

- [`README.md`](../README.md) — 本章の章構成・段階確定範囲
- [`00_方針/01_ビルド設計原則.md`](../00_方針/01_ビルド設計原則.md) — POL ID の本文
- 各サブディレクトリの `01_*.md` — CW / GM / TP / DS / PF / CS の本文
