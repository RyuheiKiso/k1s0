# 01. 対応 IMP-CODEGEN 索引

本ファイルは [`05_実装/20_コード生成設計/`](../README.md) 章配下で採番された全 `IMP-CODEGEN-*` ID を 1 ページに集約する横断索引である。各 ID から所在ファイル・対応原則・関連 ADR / DS / NFR への逆引きが可能で、PR レビュー時の影響範囲確認や、新規 ID 採番時の重複チェックを最短動線で行うために用意する。本索引は `IMP-CODEGEN-*` の正典とし、各章本文と齟齬が出た場合は本索引を改訂後に各章を更新する運用とする。

## 採番ルール

`IMP-CODEGEN-*` ID は次の規約で採番する。`10_ビルド設計/90_対応IMP-BUILD索引/` の規約と整合させ、接頭辞 → 番号レンジ → 連番の 3 段で運用する。

- 形式: `IMP-CODEGEN-<接頭辞>-<番号>`
  - 接頭辞は本章配下のサブディレクトリ単位で割り当てる（例: `BUF` = buf Protobuf、`OAS` = OpenAPI、`SCF` = Scaffold、`GLD` = Golden snapshot）
  - 番号は接頭辞内で 3 桁ゼロ埋めの連番（`010`, `011`, ...）
- 接頭辞別の番号レンジは 10 単位で予約し、欠番が出ても再利用しない
- 採番時は本索引の対応表を**先に**更新し、その後で本文ファイルに ID を埋め込む
- ID の意味（説明文）は本索引と本文ファイルで完全一致させる

接頭辞と章の対応は以下とする。

| 接頭辞 | 略称 | 所在 | 番号レンジ |
|---|---|---|---|
| `POL` | Policy | [`00_方針/01_コード生成原則.md`](../00_方針/01_コード生成原則.md) | 001 〜 009 |
| `BUF` | Buf Protobuf | [`10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md`](../10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md) | 010 〜 019 |
| `OAS` | OpenAPI Spec | [`20_OpenAPI/01_OpenAPI生成パイプライン.md`](../20_OpenAPI/01_OpenAPI生成パイプライン.md) | 020 〜 029 |
| `SCF` | Scaffold CLI | [`30_Scaffold_CLI/01_Scaffold_CLI設計.md`](../30_Scaffold_CLI/01_Scaffold_CLI設計.md) | 030 〜 039 |
| `GLD` | Golden snapshot | [`40_Golden_snapshot/01_Golden_snapshot.md`](../40_Golden_snapshot/01_Golden_snapshot.md) | 040 〜 049 |

`POL` だけは Scaffold 寄りの原則（POL-005 〜 POL-007）と全系統共通の原則（POL-001 〜 POL-004）が混在する。POL-005（golden snapshot）は文言上 Scaffold 限定だが、`40_Golden_snapshot/` で Protobuf / OpenAPI まで一般化する具体実装を規定している。

## 全 ID 一覧（接頭辞別）

### POL: コード生成原則（7 件）

| ID | 概要 |
|---|---|
| `IMP-CODEGEN-POL-001` | 契約は `src/contracts/` 単一真実源 |
| `IMP-CODEGEN-POL-002` | 生成の機械化と CI ステージ組込 |
| `IMP-CODEGEN-POL-003` | `buf breaking` による互換性検知 |
| `IMP-CODEGEN-POL-004` | 生成物 commit と「DO NOT EDIT」徹底 |
| `IMP-CODEGEN-POL-005` | Scaffold 出力の golden snapshot 検証（GLD で 3 系統に一般化） |
| `IMP-CODEGEN-POL-006` | `catalog-info.yaml` 必須生成 |
| `IMP-CODEGEN-POL-007` | テンプレート変更の SRE + Security 二重承認 |

### BUF: buf Protobuf パイプライン（8 件）

| ID | 概要 |
|---|---|
| `IMP-CODEGEN-BUF-010` | 単一 `buf.yaml` module + 言語別 `buf.gen.*.yaml` 4 分割 |
| `IMP-CODEGEN-BUF-011` | tier1 サーバーと SDK の生成先物理パス分離 |
| `IMP-CODEGEN-BUF-012` | `include_types` による internal package の SDK 除外 |
| `IMP-CODEGEN-BUF-013` | `buf breaking` FILE レベルの必須ゲート |
| `IMP-CODEGEN-BUF-014` | 生成 drift 検出スクリプトによる DO NOT EDIT 強制 |
| `IMP-CODEGEN-BUF-015` | `tools/codegen/buf.version` による CLI バージョン固定 |
| `IMP-CODEGEN-BUF-016` | v1 → v2 ディレクトリ分岐による breaking 変更経路 |
| `IMP-CODEGEN-BUF-017` | `.gitattributes` の linguist-generated 宣言 |

### OAS: OpenAPI パイプライン（8 件）

| ID | 概要 |
|---|---|
| `IMP-CODEGEN-OAS-020` | `src/contracts/openapi/v1/` 単一 yaml ディレクトリ + 3 ジェネレータ設定の物理分離 |
| `IMP-CODEGEN-OAS-021` | portal / admin / external-webhook の 3 系統限定（OpenAPI 採用範囲の境界） |
| `IMP-CODEGEN-OAS-022` | openapi-typescript / oapi-codegen / NSwag.MSBuild の言語別採用と理由 |
| `IMP-CODEGEN-OAS-023` | 生成先物理パス分離（tier3 web / tier3 BFF / tier1 external / tier2 / SDK） |
| `IMP-CODEGEN-OAS-024` | oasdiff `--fail-on ERR` の必須ゲート |
| `IMP-CODEGEN-OAS-025` | `tools/codegen/verify-openapi-drift.sh` による DO NOT EDIT 強制 |
| `IMP-CODEGEN-OAS-026` | `tools/codegen/openapi.versions` による 4 CLI バージョン固定 |
| `IMP-CODEGEN-OAS-027` | v1 → v2 ディレクトリ分岐による breaking 変更経路 |

### SCF: Scaffold CLI（8 件）

| ID | 概要 |
|---|---|
| `IMP-CODEGEN-SCF-030` | Rust 実装の Scaffold CLI（`src/platform/scaffold/` crate） |
| `IMP-CODEGEN-SCF-031` | Backstage Software Template 互換の `template.yaml` 採用 |
| `IMP-CODEGEN-SCF-032` | tier2 / tier3 テンプレート配置分離（`src/tier2/templates/` / `src/tier3/templates/`） |
| `IMP-CODEGEN-SCF-033` | `catalog-info.yaml` の自動生成と CODEOWNERS 連動 owner 推論 |
| `IMP-CODEGEN-SCF-034` | SRE + Security 二重承認の branch protection 強制 |
| `IMP-CODEGEN-SCF-035` | golden snapshot 検証と `UPDATE_GOLDEN=1` 承認プロセス |
| `IMP-CODEGEN-SCF-036` | テンプレート semver バージョニング（`k1s0.io/template-version`） |
| `IMP-CODEGEN-SCF-037` | リリース時点 の Backstage UI 統合経路（Custom action 化） |

### GLD: Golden snapshot（8 件）

| ID | 概要 |
|---|---|
| `IMP-CODEGEN-GLD-040` | `tests/codegen/golden-{input,output}/` の物理配置と本番 contracts からの分離 |
| `IMP-CODEGEN-GLD-041` | Protobuf / OpenAPI / Scaffold 3 系統の最小代表サンプル原則 |
| `IMP-CODEGEN-GLD-042` | `run-golden-snapshot.sh` の exit code 3 値設計（0 / 1 / 2）と CI ラベル誘導 |
| `IMP-CODEGEN-GLD-043` | `update-golden-snapshot.sh` の物理分離（誤上書き防止） |
| `IMP-CODEGEN-GLD-044` | snapshot 更新 PR の記載要件と CODEOWNERS 必須レビュー |
| `IMP-CODEGEN-GLD-045` | reusable workflow `codegen-golden-snapshot` の trigger 条件と月次 schedule |
| `IMP-CODEGEN-GLD-046` | `normalize.sh` による非決定要素の正規化ポリシー |
| `IMP-CODEGEN-GLD-047` | drift 検出と golden snapshot の役割分担文書化（POL-004 / POL-005 との関係、CLI バージョン固定 BUF-015 / OAS-026 との一体運用） |

## 採番済み件数まとめ

| 接頭辞 | 件数 | 残レンジ | 次番 |
|---|---|---|---|
| `POL` | 7 | 002 件（008-009） | `IMP-CODEGEN-POL-008` |
| `BUF` | 8 | 002 件（018-019） | `IMP-CODEGEN-BUF-018` |
| `OAS` | 8 | 002 件（028-029） | `IMP-CODEGEN-OAS-028` |
| `SCF` | 8 | 002 件（038-039） | `IMP-CODEGEN-SCF-038` |
| `GLD` | 8 | 002 件（048-049） | `IMP-CODEGEN-GLD-048` |
| **合計** | **39** | **10** | — |

各接頭辞のレンジが残り 2 件まで埋まっており、追加採番が見込まれる場合はレンジ拡張（例: `BUF-010` 〜 `029` への 2 倍化）を ADR 起票で行う。レンジを跨ぐ単純連番は採番ミスの温床となるため、レンジ拡張は明示的な意思決定として記録する。

## 対応 ADR 逆引き

`IMP-CODEGEN-*` から参照される ADR を逆引きで一覧化する。各 ADR がどの IMP-CODEGEN ID に影響するかを把握する時に使う。

| ADR | 影響する IMP-CODEGEN ID |
|---|---|
| [ADR-TIER1-002](../../../02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md)（Protobuf gRPC 統一） | POL-001〜004, BUF-010〜017, OAS-020〜022（HTTP 例外境界の規定） |
| [ADR-TIER1-003](../../../02_構想設計/adr/ADR-TIER1-003-language-opacity.md)（内部言語不可視） | POL-001, BUF-011〜012, OAS-023, SCF-032 |
| [ADR-DIR-001](../../../02_構想設計/adr/ADR-DIR-001-contracts-elevation.md)（contracts 昇格） | POL-001, BUF-010〜011, OAS-020, GLD-040 |
| [ADR-DIR-003](../../../02_構想設計/adr/ADR-DIR-003-sparse-checkout-cone-mode.md)（sparse cone） | BUF-011, OAS-023, GLD-040（cone 整合確認） |
| [ADR-CICD-001](../../../02_構想設計/adr/ADR-CICD-001-argocd.md)（CI 構成） | POL-002, BUF-013〜014, OAS-024〜025, SCF-034〜035, GLD-042, GLD-045 |

新規 ADR 起票時は本逆引き表と本文両方を同期更新する。

## 対応 DS-SW-COMP 逆引き

| DS-SW-COMP | 影響する IMP-CODEGEN ID |
|---|---|
| DS-SW-COMP-122（contracts → 4 言語生成 / SDK） | POL-001, BUF-010〜017, OAS-020〜027, GLD-040〜047 |
| DS-SW-COMP-129 / 130（SDK 配置と利用境界 / 契約配置） | BUF-010〜012, OAS-020〜023, GLD-040 |
| DS-SW-COMP-140（外部 IF 設計） | OAS-020〜022（external-webhook の境界） |

## 対応 NFR 逆引き

| NFR | 影響する IMP-CODEGEN ID |
|---|---|
| [NFR-H-INT-001](../../../03_要件定義/30_非機能要件/H_セキュリティ.md)（署名付きアーティファクト） | BUF-014, OAS-025（生成物 commit と検証経路） |
| [NFR-C-MNT-003](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（API 互換方針） | POL-003, BUF-013, BUF-016, OAS-024, OAS-027 |
| [NFR-C-MGMT-001](../../../03_要件定義/30_非機能要件/C_運用保守性.md)（設定 Git 管理） | POL-001, POL-004, 全 ID（buf.yaml / openapi yaml / template.yaml の commit 必須） |
| [NFR-Q-PROD-002](../../../03_要件定義/30_非機能要件/Q_品質保証.md)（CI 自動化） | POL-002, BUF-013〜014, OAS-024〜025, SCF-034〜035, GLD-042〜045 |

## 上位索引との連携

本索引は [`05_実装/20_コード生成設計/`](../README.md) 章内の局所索引である。`IMP-CODEGEN-*` 全件を含むより上位の索引は以下に置かれる予定で、本索引はそこへ集約される位置付けとなる。

- [`05_実装/99_索引/`](../../99_索引/) — `IMP-*` 全接頭辞（DIR / BUILD / CODEGEN / CI / DEP / DEV / OBS / REL / SUP / SEC / POL / DX / TRACE）の横断索引
- [`04_概要設計/80_トレーサビリティ/01_設計ID索引.md`](../../../04_概要設計/80_トレーサビリティ/01_設計ID索引.md) — 設計 ID と実装 ID の突合せ

新規章追加・新規接頭辞追加・新規 ID 採番のたびに本索引と上位索引の両方を同期更新する。同期忘れは PR レビュー（[`docs-review-checklist`](../../../00_format/review_checklist.md)）の必須チェック項目とする。

## 関連章 / 参照

- [`README.md`](../README.md) — 本章の章構成・段階確定範囲
- [`00_方針/01_コード生成原則.md`](../00_方針/01_コード生成原則.md) — POL ID の本文
- [`10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md`](../10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md) — BUF ID の本文
- [`20_OpenAPI/01_OpenAPI生成パイプライン.md`](../20_OpenAPI/01_OpenAPI生成パイプライン.md) — OAS ID の本文
- [`30_Scaffold_CLI/01_Scaffold_CLI設計.md`](../30_Scaffold_CLI/01_Scaffold_CLI設計.md) — SCF ID の本文
- [`40_Golden_snapshot/01_Golden_snapshot.md`](../40_Golden_snapshot/01_Golden_snapshot.md) — GLD ID の本文
- [`../10_ビルド設計/90_対応IMP-BUILD索引/01_対応IMP-BUILD索引.md`](../../10_ビルド設計/90_対応IMP-BUILD索引/01_対応IMP-BUILD索引.md) — 隣接章の対応索引（書式統一の参照元）
