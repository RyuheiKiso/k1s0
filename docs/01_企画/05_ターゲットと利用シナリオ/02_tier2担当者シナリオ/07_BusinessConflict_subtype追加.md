---
id: plan.tier2.scenario_business_conflict_subtype
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D]
  proof_classes: []
---

# BusinessConflict subtype 追加

## 一文方針

並行編集シナリオの新 conflict subtype を追加する際、subtype 定義・Apicurio 互換検査・tier3 向け subtype-UI binding contract 宣言・PII 非含有確認・aggregate 実装・両層の test green を順守してから merge する。

> 朝 9 時、本社 IT 室の tier2 担当者（中堅級）が GitHub PR で `dual_field_lost_update` subtype 追加の要求チケットを確認する。手元には Apicurio Registry UI、Mattermost 越しに tier3 担当者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: 新 BusinessConflict subtype の定義から Apicurio 互換検査・PII 非含有確認・tier3 UI binding contract 宣言・両層テスト green まで一貫して通す

## 現状業務での痛み

- 並行編集 conflict の新 subtype が ad hoc に実装されて subtype-UI binding 仕様が不明確になり、tier3 担当者が UI を実装できない状態になる
- PII redaction 確認が手動で属人的になり、recovery_hints に個人情報が混入するリスクがある
- Apicurio compatibility check なしで subtype が追加され、既存 consumer が新 subtype を受け取れずに UI 崩壊が発生する
- Playwright E2E test が整備されておらず、UI 側での conflict 処理の動作確認が手動になっている

## k1s0 でこう変わる

- subtype-UI binding 仕様を tier2 側で先行して宣言し、tier3 担当者が contract を参照して UI 実装できる体制を整える
- PII 非含有確認を audit / redaction 規約に照らして実施し、PII 混入を設計段階で排除する
- Apicurio compatibility check が CI gate となり、既存 subtype との後方互換性を merge 前に担保する
- Playwright E2E test（tier3）と integration test（tier2）の両方 green が merge 条件となる

## Trigger（発火条件）

並行編集シナリオの新しい conflict subtype（stale_write / lost_update / supersede / concurrent_edit 以外）を追加する必要が生じた時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「2 名が同時に同じ受注明細の数量と納期を編集し、lost_update 型の conflict が頻発した」
- 典型きっかけ 2: 「`dual_field_lost_update` という新 subtype を追加し、数量（quantity）と納期（delivery_date）の同時変更を検出する仕組みを実装した」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: tier3 担当者（subtype-UI binding 実装・Playwright E2E test）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | GitHub PR / Apicurio Registry UI | subtype 定義 / conflict 検出ロジック実装 / PII 非含有確認 |
| 関与（tier3）| 中堅 | 本社 / リモート | GitHub PR / Mattermost `#tier2-ops` | subtype-UI binding 実装 / Playwright E2E test |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | PR レビュー / sign-off（author 不可） |

## 個人 KPI / 達成感

- Apicurio compatibility check green（既存 subtype との後方互換）
- PII が recovery_hints に含まれていないことの確認完了
- Playwright E2E test（tier3）と integration test（tier2）両方 green
- dual reviewer 応答時間 ≤ 48h

## 工数 / 関与人数 / コスト感

- 初回: 2〜3 日（subtype 定義・UI binding 宣言・PII 確認・conflict 検出ロジック実装・両層テスト）、関与 4 名（主役 + tier3 担当者 + dual reviewer 2 名）
- 平常（recovery_hints / UI binding 変更のみ）: 1 日、関与 3〜4 名
- 失敗時（Apicurio fail / PII 混入 / Playwright fail）: +1 日、関与 4〜5 名（PII 混入時は + security 担当者）

## 前提

- 既存の BusinessConflict subtype（stale_write / lost_update / supersede / concurrent_edit）が Apicurio Registry に登録済みであること
- tier3 との [subtype-UI binding](../../../03_概要設計/04_tier3設計方針/13_業務エラーUX.md)（BusinessConflict の subtype と tier3 UI の 1:1 対応仕様）contract の仕組みが確立済みであること
- PII redaction 規約が確立済みであること

## 流れ

1. 新 subtype の定義（発生条件 / recovery 手順 / user 向け recovery_hints）を draft する
2. Domain Event に BusinessConflict subtype を追加し Apicurio compatibility check を通過させる
3. tier3 の UI 側と 1:1 対応する subtype-UI binding 仕様を tier2 側で宣言する（tier3 担当が UI を実装するための contract）
4. PII が recovery_hints に含まれないことを audit / redaction 規約に照らして確認する
5. aggregate の conflict 検出ロジックを実装し、3-way merge / field-level rebase / silent toast のいずれかを業務要件に基づいて選択する
   - 同一 field の競合（同時更新）→ 3-way merge UI（ユーザに選択させる）
   - 異 field の競合（A: 数量、B: 納期）→ field-level rebase（自動的に双方を保持）
   - 後続 op が前 op を supersede（同一 actor の連続操作）→ silent toast（ユーザ操作不要）
   - presence indicator で競合予防（concurrent_edit）→ user choice UI
6. Playwright E2E test（tier3 側）と integration test（tier2 側）を両方 green にしてから merge する
7. dual reviewer sign-off を得てから merge する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | subtype 定義 draft・recovery_hints PII 確認 | `dual_field_lost_update 定義 draft / recovery_hints: PII なし確認済` |
| 1 日 | tier2 担当者 | Apicurio push / compatibility check・UI binding 宣言・conflict 検出ロジック実装 | `Apicurio green / UI binding contract 宣言済 / conflict 検出ロジック実装完了` |
| 1.5 日 | tier3 担当者 | subtype-UI binding 実装・Playwright E2E test 作成 | `UI 実装完了 / Playwright E2E: green` |
| 2 日 | tier2 担当者 | integration test 実行・両層 green 確認 | `integration test: green / Playwright: green` |
| 3 日 | dual reviewer | sign-off | `dual sign-off 完了` |

## 業界 9 業務との紐付け

- **受注**: 受注明細の数量・納期の同時編集（`dual_field_lost_update`）が最も頻発する業務であり、新 subtype 追加の直接的な動機となる。
- **在庫**: 在庫数量の並行更新競合が silent toast または 3-way merge UI で適切に処理されることで、在庫データの整合性が保たれる。
- **FA 生産指示・設備操作**: 生産指示の同時変更競合が presence indicator（concurrent_edit subtype）で予防される。

## 関連適合仕様 / 関連 OSS

- [スキーマ進化適合仕様](../../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- Apicurio Registry（BusinessConflict subtype schema の互換性管理）
- Playwright（tier3 側 E2E test）

## 期待結果 / 観測指標

- Apicurio compatibility check が green になること
- tier3 担当者が subtype-UI binding contract を参照して UI 実装できること
- PII が recovery_hints に含まれていないことが確認されること
- Playwright E2E test（tier3）と integration test（tier2）が両方 green になること

## 失敗時の挙動 / escalation

- **Apicurio compatibility check fail**: schema 変更が既存 subtype との後方互換性を破壊していると判断し設計を見直す。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `business-conflict-subtype-procedure`
- **PII 混入 / audit chain 欠落 fail**: security 担当に escalation し即時修正。audit chain 欠落は compliance incident として ops 担当者と security 担当者に即時 escalate。escalate 先: Mattermost `#security-incident`（SLA: PII 混入 → 即時修正、audit chain 欠落 → 1h 以内）。runbook: Backstage `business-conflict-subtype-procedure`
- **Playwright E2E test fail**: tier3 担当者と連携して subtype-UI binding contract の齟齬を解消する。escalate 先: Mattermost `#tier2-incident`（SLA: 48h 以内に是正 PR 提出）。runbook: Backstage `business-conflict-subtype-procedure`

## 失敗パターン (anti-pattern)

- **subtype-UI binding 仕様を tier3 担当者任せにする**: tier3 担当者が独自の解釈で UI を実装し、tier2 の conflict 検出ロジックと不整合が生じる。tier2 側で先行して subtype-UI binding contract を宣言してから tier3 に UI 実装を依頼する順序を徹底する。
- **recovery_hints の PII 確認を省略**: 受注明細のフィールド値（個人名・金額等）が recovery_hints に混入し、compliance incident になる。PII 非含有確認を subtype 定義 draft の必須ステップとして実施し、audit / redaction 規約に照らして確認する。
- **Playwright E2E test なしで「integration test だけ green」で merge**: tier2 側の logic は正しくても tier3 の UI 表示が conflict subtype を正しくハンドリングしていないことが本番で発覚する。Playwright E2E test と integration test の両方 green を merge の必須条件とする。

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [並行編集シナリオ](../README.md)
