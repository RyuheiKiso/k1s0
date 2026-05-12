---
id: plan.overview.scenario_ops_sre_50pc_rule_toil
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.ops.ops_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D]
  proof_classes: []
---

# SRE_50pc_rule_toil削減

## 一文方針

月次 toil 比率 review で SRE 50% rule 維持を確認し、toil > 50% を超えている担当者の自動化 backlog を更新して Tekton pipeline 追加 PR を起票する。

> 月末金曜、ops 担当者が月次 toil review のスプレッドシートを開く。先月の ops 担当者 4 名の toil 比率を集計すると、担当者 A が 58%、担当者 B が 62% で SRE 50% rule を超過している。主な toil 源は「手動 runbook 実行」と「alert noise による手動確認」だ。ops 担当者は自動化 backlog に 2 件の Tekton pipeline 追加タスクを登録し、tier1 担当者に実装 PR の作成を依頼する。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: 月次 toil 比率を確認し、50% 超過担当者の自動化 backlog を更新して Tekton pipeline 追加 PR を起票する

## 現状業務での痛み

- toil 比率を計測する仕組みがなく、担当者が実感で「忙しい」と感じていても数値化できない
- 自動化 backlog が存在しないため、toil 削減の優先順位付けができない
- Tekton pipeline の追加は開発者に依頼しなければならないが、依頼先と手順が明確でない
- SRE 50% rule の超過が常態化しても、組織として改善アクションが取られていない

## k1s0 でこう変わる

- 月次 toil 比率が計測・記録され、担当者・チーム単位で数値が可視化される
- 自動化 backlog が GitHub Project として管理され、優先順位付きで toil 削減が進む
- Tekton pipeline 追加 PR の起票手順が Backstage runbook に定義され、依頼フローが明確になる
- SRE 50% rule 超過の担当者が自動で特定され、Mattermost `#ops-sre-health` に週次 digest が送られる

## Trigger

月次 toil review 日程到来時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「月末の toil review で担当者 2 名が SRE 50% rule 超過と判明」「SRE 50% rule 超過検知 alert が Mattermost に届いた」
- 頻度根拠: SRE 50% rule は月次 cadence での review が SRE best practice

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | toil ratio dashboard / GitHub Project | toil 比率集計・自動化 backlog 更新・Tekton pipeline PR 起票依頼 |
| tier1 担当者 | シニア | 本社 IT 室 / リモート | GitHub Project | Tekton pipeline 追加 PR 実装（escalation 受け） |

## 個人 KPI / 達成感

- 全 ops 担当者の toil 比率 50% 以下維持率: 月次 100% が目標
- 自動化 backlog の消化率: 月次 2 件以上
- SRE 50% rule 超過担当者数の 3 ヶ月移動平均: 逓減傾向

## 工数 / 関与人数 / コスト感

- 月次 toil 比率集計: 30〜60 分
- 自動化 backlog 更新: 30 分
- Tekton pipeline PR 起票依頼: 15 分
- 関与人数: 2〜4 名（ops チーム + tier1）

## 前提

- 月次 toil 比率を計測・記録する仕組みが整備されている（ClickHouse / スプレッドシート等）
- 自動化 backlog が GitHub Project として存在する
- Tekton pipeline 追加 PR の起票手順が Backstage runbook に定義されている

## 流れ

1. **toil 比率集計**: 月次 toil review 日程到来時に、ops 担当者全員の前月 toil 比率を集計する
2. **50% 超過担当者特定**: toil > 50% の担当者を特定し、主要 toil 源（手動 runbook / alert noise / 定型作業）を分類する
3. **toil 源分析**: 各 toil 源について自動化の余地を評価する
4. **自動化 backlog 更新**: 自動化可能なタスクを GitHub Project の自動化 backlog に追加し、優先順位と担当者を設定する
5. **Tekton pipeline 追加 PR 起票依頼**: pipeline 自動化が有効な toil 源について、tier1 担当者に Tekton pipeline 追加 PR の実装を依頼する
6. **`toil_reduction_log.md` 更新**: 今月の toil 比率・超過担当者・追加した自動化 backlog を Backstage TechDocs に記録する
7. **Mattermost 報告**: Mattermost `#ops-sre-health` に月次 review 結果を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | ops 担当者 | toil 比率集計開始 | — |
| T+1h | ops 担当者 | 50% 超過担当者・toil 源特定完了 | — |
| T+1.5h | ops 担当者 | 自動化 backlog 更新（GitHub Project） | — |
| T+2h | ops 担当者 | tier1 に Tekton pipeline PR 起票依頼 | Mattermost DM: 「Tekton pipeline: alert-auto-ack の実装依頼」 |
| T+2.5h | ops 担当者 | toil_reduction_log 更新・Mattermost 報告 | 「[SRE toil review] 担当者 A: 58%、担当者 B: 62% 超過。自動化 backlog 2 件追加」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信 alert noise の自動化が最大の toil 削減対象 |
| SCADA テレメトリ収集 | 中 | テレメトリ監視の定型確認を自動化 |
| FA 生産指示 | 中 | 生産指示配信の定期確認 toil を削減 |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- 関連 OSS: Tekton（pipeline 自動化）/ Mattermost（通知）/ Backstage TechDocs（toil_reduction_log）/ GitHub Project（backlog）

## 期待結果 / 観測指標

- 全 ops 担当者の toil 比率が月次 50% 以下に維持されている
- `toil_reduction_log.md` に月次の集計結果と自動化 backlog 追加が記録されている
- 自動化 backlog の月次消化件数が 2 件以上

## 失敗時の挙動 / escalation

- **toil 比率が 3 ヶ月連続 50% 超過**: ops チームリーダーと tech lead に自動 escalation。自動化投資の優先度を上げるよう経営判断を促す
- **Tekton pipeline 追加 PR が依頼から 2 週間以内に作成されない**: ops 担当者が tier1 チームリーダーに優先度上げを依頼する
- **toil 比率の計測データが取得できない**: 手動集計にフォールバックし、計測自動化を次月の自動化 backlog 第 1 位に登録する

## 失敗パターン (anti-pattern)

1. **toil 比率を「体感」で報告する**: 数値根拠のない改善優先度付けになり、高 toil タスクが放置される
2. **自動化 backlog を作るだけで消化しない**: backlog が肥大化し、形骸化する。月次で消化件数を追跡する
3. **Tekton pipeline 追加を「後で」に先送りする**: toil が蓄積し、担当者の疲弊とオンコール対応品質の低下につながる

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [runbook整備_Backstage_TechDocs](04_runbook整備_Backstage_TechDocs.md)
- [24h_oncall_fatigue_budget管理](14_24h_oncall_fatigue_budget管理.md)
