# 01. 対応 IMP-CI-E2E 索引

本ファイルは 35_e2e_test_design 章で採番される実装 ID（IMP-CI-E2E-\*）を索引化する。各 ID の内容、対応 ADR、配置ファイル、依存関係を一覧化することで、後続の実装段階で「どの ID が何を担うか」を直読可能にする。

## 本ファイルの位置付け

35_e2e_test_design 章の各節で採番した IMP-CI-E2E-001 〜 IMP-CI-E2E-016 を、本ファイル 1 箇所で一覧化する。実装 PR が「この PR は IMP-CI-E2E-NNN を実装する」と特定 ID で参照する経路、`tools/audit/run.sh` が ID coverage を集計する経路、`docs/05_実装/99_索引/00_IMP-ID一覧/` から本ファイルへの link 経路の 3 用途を担う。

## IMP-CI-E2E-\* 一覧

リリース時点で採番済の 16 ID。今後の追加 ID は本一覧に追記する形で確定する（採番は IMP-CI-E2E-017 から連番）。

| ID | 内容 | 配置ファイル | 主要対応 ADR | 状態 |
|---|---|---|---|---|
| IMP-CI-E2E-001 | owner / user 二分原則と判定基準 | `00_方針/01_owner_user_責務分界.md` | ADR-TEST-008 | Proposed |
| IMP-CI-E2E-002 | owner suite 環境契約（multipass + kubeadm + Cilium + Longhorn + MetalLB + フルスタック） | `10_owner_suite/01_環境契約.md` | ADR-TEST-008 / INFRA-001 / NET-001 / STOR-001 / STOR-002 | Proposed |
| IMP-CI-E2E-003 | user suite 環境契約（kind + minimum stack + 任意 stack opt-in） | `20_user_suite/01_環境契約.md` | ADR-TEST-008 / NET-001 | Proposed |
| IMP-CI-E2E-004 | tests/e2e/owner/ 8 部位 + helpers + go.mod 構造 | `10_owner_suite/02_ディレクトリ構造.md` | ADR-TEST-008 / 009 | Proposed |
| IMP-CI-E2E-005 | tests/e2e/user/ 2 部位 + helpers + go.mod 構造 | `20_user_suite/02_ディレクトリ構造.md` | ADR-TEST-008 / 010 | Proposed |
| IMP-CI-E2E-006 | make e2e-owner-* 8 target 実装契約 | `10_owner_suite/03_Makefile_target.md` | ADR-TEST-008 | Proposed |
| IMP-CI-E2E-007 | make e2e-user-{smoke, full} 実装契約 | `20_user_suite/03_Makefile_target.md` | ADR-TEST-008 | Proposed |
| IMP-CI-E2E-008 | 観測性 5 検証実装契約（5 ファイル + 5 helper client + baseline 管理規約） | `10_owner_suite/04_観測性5検証.md` | ADR-TEST-009 | Proposed |
| IMP-CI-E2E-009 | _reusable-e2e-user.yml 構造 | `20_user_suite/04_CI戦略.md` | ADR-TEST-008 / CICD-001 | Proposed |
| IMP-CI-E2E-010 | pr.yml + nightly.yml への user e2e 統合経路 | `20_user_suite/04_CI戦略.md` | ADR-TEST-008 / TEST-007 | Proposed |
| IMP-CI-E2E-011 | 4 言語 fixtures 対称 API（5 領域 × 4 言語） | `30_test_fixtures/01_4言語対称API.md` | ADR-TEST-010 / TIER1-001 | Proposed |
| IMP-CI-E2E-012 | SDK + test-fixtures の同 module / 同 version 運用規約 | `30_test_fixtures/02_versioning.md` | ADR-TEST-010 / TEST-011 | Proposed |
| IMP-CI-E2E-013 | mock builder 段階展開規約（3 → 6 → 12 service × 4 言語） | `30_test_fixtures/03_mock_builder段階展開.md` | ADR-TEST-010 | Proposed |
| IMP-CI-E2E-014 | cut.sh の release tag ゲート拡張（step 4-5 + tag メッセージ埋め込み） | `40_release_tag_gate/01_cut_sh_拡張.md` | ADR-TEST-011 / TEST-001 | Proposed |
| IMP-CI-E2E-015 | tests/.owner-e2e/ ディレクトリ + git LFS 12 ヶ月管理 | `40_release_tag_gate/02_artifact_保管.md` | ADR-TEST-011 / TEST-003 | Proposed |
| IMP-CI-E2E-016 | owner-e2e-results.md entry フォーマット規約（必須フィールド + cut.sh parse 経路） | `40_release_tag_gate/03_owner_e2e_results_template.md` | ADR-TEST-011 / OPS-001 | Proposed |

## ID の意味的グルーピング

16 ID は以下 4 グループに整理できる。各グループは 4 ADR の対応関係を示す。

### グループ 1: 二分構造の原則（ADR-TEST-008 系列）

- IMP-CI-E2E-001（責務分界）
- IMP-CI-E2E-002 / 003（環境契約）
- IMP-CI-E2E-004 / 005（ディレクトリ構造）
- IMP-CI-E2E-006 / 007（Makefile target）
- IMP-CI-E2E-009 / 010（CI 統合）

owner / user 二分の物理化に必要な 9 ID。これらは ADR-TEST-008 の決定を実装段階に展開する。

### グループ 2: 観測性 5 検証（ADR-TEST-009 系列）

- IMP-CI-E2E-008（5 検証実装契約）

ADR-TEST-009 を実装段階に展開する単一 ID。helper / baseline / 5 検証の段階展開を内包する。

### グループ 3: test-fixtures（ADR-TEST-010 系列）

- IMP-CI-E2E-011（4 言語対称 API）
- IMP-CI-E2E-012（versioning）
- IMP-CI-E2E-013（mock builder 段階展開）

利用者 DX を支える 3 ID。SDK 同梱 fixtures の核心。

### グループ 4: release tag ゲート（ADR-TEST-011 系列）

- IMP-CI-E2E-014（cut.sh 拡張）
- IMP-CI-E2E-015（artifact 保管）
- IMP-CI-E2E-016（results.md template）

owner full の代替保証経路を実装する 3 ID。release tag に owner full PASS を物理的に紐付ける。

## 状態遷移

各 ID の状態は以下を取る。リリース時点では全 16 ID が Proposed。

| 状態 | 意味 |
|---|---|
| Proposed | 設計書に記述済、実装着手前 |
| InProgress | 実装中（PR 起票済） |
| Implemented | 実装完了、CI で機械検証 PASS |
| Verified | owner full / user full で実走確認、`docs/40_運用ライフサイクル/{owner,user}-e2e-results.md` に記録 |

状態遷移は実装 PR の merge と `tools/audit/run.sh` の集計で更新する。本一覧の状態列は人手で更新する（PR の commit 時に同期）。

## 依存関係

実装着手の依存順は以下。上位の ID から実装する必要がある。

```text
IMP-CI-E2E-001 (責務分界)
  ├── IMP-CI-E2E-002 (owner 環境契約)
  │     └── IMP-CI-E2E-004 (owner ディレクトリ)
  │           └── IMP-CI-E2E-006 (owner Makefile target)
  │                 ├── IMP-CI-E2E-008 (観測性 5 検証)
  │                 └── IMP-CI-E2E-014 (cut.sh 拡張、IMP-CI-E2E-015 + 016 と並列)
  └── IMP-CI-E2E-003 (user 環境契約)
        └── IMP-CI-E2E-005 (user ディレクトリ)
              └── IMP-CI-E2E-007 (user Makefile target)
                    └── IMP-CI-E2E-009 (_reusable-e2e-user.yml)
                          └── IMP-CI-E2E-010 (pr.yml + nightly.yml)

IMP-CI-E2E-011 (fixtures 4 言語 API)  ← user / owner 両方の前提
  ├── IMP-CI-E2E-012 (versioning)
  └── IMP-CI-E2E-013 (mock builder 段階展開)

IMP-CI-E2E-015 (artifact 保管) + IMP-CI-E2E-016 (results.md template) ← cut.sh の前提
  └── IMP-CI-E2E-014 (cut.sh 拡張)
```

並列着手可能なグループ:

- グループ 1 の owner 系（002 → 004 → 006 → 008）と user 系（003 → 005 → 007 → 009 → 010）は独立並列
- グループ 3（fixtures、011-013）は他グループと並列着手可能（user suite が完成する前段で fixtures 着手）
- グループ 4（release gate、014-016）は owner suite 完成（IMP-CI-E2E-006）後に着手

## 上位索引との連携

本一覧は `docs/05_実装/99_索引/00_IMP-ID一覧/README.md` から link で参照される（採用初期で索引整備時に追加）。`docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` には `IMP-CI-E2E-*` 系列の参照リンクを追加する（リリース時点で対応）。

## 対応 ADR / 関連設計

- ADR-TEST-008 / 009 / 010 / 011 — 本一覧の起源
- `README.md`（同章）— 章全体索引
- `docs/05_実装/99_索引/00_IMP-ID一覧/`（採用初期で整備）— IMP ID 全系列横断索引
- `docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` — IMP-CI 全系列の章内索引
