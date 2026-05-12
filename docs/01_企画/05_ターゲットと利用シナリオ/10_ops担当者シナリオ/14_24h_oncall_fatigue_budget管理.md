---
id: plan.overview.scenario_ops_24h_oncall_fatigue_budget
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

# 24h_oncall_fatigue_budget管理

## 一文方針

24h 連続当番禁止 (fatigue budget) の状況を月次で確認し、連続 on-call 超過の担当者に対して rotation 調整を行い、長期的な SRE 50% rule 遵守を維持する。

> 月末の水曜、ops 担当者が on-call rotation の月次 fatigue budget review を開く。先月の rotation ログを確認すると、担当者 C が 9 日間連続で on-call に入っており、24h 連続当番禁止ルールを複数回超過していたことが分かる。ops 担当者は次月の rotation を調整し、担当者 C の on-call 間隔を最低 24h 空けるよう再設定する。SRE 50% rule との整合も確認し、`oncall_rotation_plan.lock.yaml` を更新する。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: 月次 fatigue budget review で 24h 連続当番禁止ルールを確認し、rotation 調整で SRE 50% rule 遵守を長期的に維持する

## 現状業務での痛み

- 24h 連続当番禁止ルールが定義されていても、rotation の組み方が属人的で超過が頻発する
- on-call fatigue による担当者の疲弊が incident 対応品質の低下につながっていても、数値化されていない
- 月次 review が行われず、疲弊が蓄積した担当者が退職という形で顕在化してから対応が始まる
- SRE 50% rule と on-call rotation の整合性を確認する機会がなく、toil が集中する担当者が生まれる

## k1s0 でこう変わる

- `oncall_rotation_plan.lock.yaml` が月次 fatigue budget review の構造化記録として機能する
- 24h 連続当番超過が自動検知され、Mattermost `#ops-sre-health` に週次 alert として通知される
- on-call rotation の自動スケジューリングツールが 24h 間隔を強制し、ルール違反を構造的に防止する
- SRE 50% rule の toil 比率と on-call 負荷を統合したダッシュボードで、担当者単位の持続可能性が可視化される

## Trigger

月次 fatigue budget review 日程到来時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「月末の fatigue budget review で担当者 C が 24h 連続当番禁止ルールを超過していた」「ops 担当者 1 名の退職申し出を受け、rotation 再設計が必要になった」
- 頻度根拠: SRE 運用規律として月次 cadence での review が必要。人員変動は不定期で発生

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | oncall_rotation_plan.lock.yaml / SRE health dashboard | fatigue budget 確認・rotation 調整・YAML 更新 |
| ops チームリーダー | シニア | 本社 IT 室 / リモート | Mattermost #ops-sre-health | 重大超過時の承認・人員補充判断 |

## 個人 KPI / 達成感

- 24h 連続当番禁止ルール超過件数: 月次 0 件が目標
- 月次 fatigue budget review 実施率: 100%
- `oncall_rotation_plan.lock.yaml` の月次更新率: 100%

## 工数 / 関与人数 / コスト感

- 月次 fatigue budget review: 30〜60 分
- rotation 調整: 30〜60 分
- 関与人数: 1〜2 名（ops + 必要時 ops チームリーダー）

## 前提

- on-call rotation ログが `oncall_rotation_plan.lock.yaml` または rotation 管理ツールで記録されている
- 24h 連続当番禁止ルールが `escalation_policy.lock.yaml` に定義されている
- ops 担当者の月次 toil 比率データが取得できる状態になっている

## 流れ

1. **rotation ログ確認**: 前月の on-call rotation ログを確認し、各担当者の on-call 連続日数を集計する
2. **24h 連続当番超過確認**: 24h 連続当番禁止ルール（on-call shift 間に最低 24h の空き）を超過した担当者を特定する
3. **SRE 50% rule との整合確認**: 月次 toil 比率（シナリオ 08 のデータ）と on-call 負荷を照合し、特定担当者への集中がないか確認する
4. **次月 rotation 設計**: 超過が発生しないよう次月の rotation を設計し直す。担当者の不在予定（休暇 / 出張）も考慮する
5. **`oncall_rotation_plan.lock.yaml` 更新**: 次月の rotation 計画を YAML に記録し、GitHub commit で証跡を残す
6. **重大超過の escalation（必要時）**: 複数担当者が継続的に超過している場合は ops チームリーダーに escalation し、人員補充の判断を促す
7. **Mattermost 報告**: Mattermost `#ops-sre-health` に月次 review 結果を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | ops 担当者 | rotation ログ確認・超過担当者特定 | — |
| T+0.5h | ops 担当者 | SRE 50% rule との整合確認 | — |
| T+1h | ops 担当者 | 次月 rotation 設計 | — |
| T+1.5h | ops 担当者 | oncall_rotation_plan.lock.yaml 更新・GitHub commit | — |
| T+2h | ops 担当者 | Mattermost 報告 | 「[fatigue budget] 月次 review 完了。担当者 C 超過: rotation 再調整済み」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信 on-call の疲弊は alert 対応品質に直結 |
| FA 生産指示 | 中 | 生産指示障害の夜間 on-call を持続可能な rotation で維持 |
| SCADA テレメトリ収集 | 中 | SCADA テレメトリ監視の on-call 負荷分散 |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- 関連 OSS: Mattermost（通知）/ Backstage（rotation ドキュメント）/ GitHub（oncall_rotation_plan.lock.yaml）

## 期待結果 / 観測指標

- 全 ops 担当者の 24h 連続当番禁止ルール遵守率: 月次 100% が目標
- `oncall_rotation_plan.lock.yaml` に月次更新が記録されている
- SRE 50% rule と on-call 負荷の整合が確認されている

## 失敗時の挙動 / escalation

- **複数担当者が 3 ヶ月連続で 24h 連続当番超過**: ops チームリーダーに自動 escalation。人員補充の判断を経営に提言する
- **rotation 調整後も 24h 空きが確保できない（人員不足）**: 超過している担当者の on-call 負荷を一時的に他の担当者に移管し、人員補充を最優先で経営に依頼する
- **`oncall_rotation_plan.lock.yaml` の更新が漏れる**: 次月の rotation が前月のままになり、24h 超過が再発する。月次 review の完了条件として YAML 更新を必須とする

## 失敗パターン (anti-pattern)

1. **担当者が「自分は大丈夫」と申告し超過を隠す**: fatigue budget review では主観的な「大丈夫」ではなく rotation ログの客観データで判断する
2. **incident が多い月は fatigue budget review を後回しにする**: incident が多い月こそ疲弊が蓄積しており、review が最も重要。incident 収束後 72 時間以内に review を実施する
3. **SRE 50% rule の toil 比率と on-call 負荷を別々に管理する**: 一方が改善しても他方が悪化するケースを見落とすリスクがある。統合ダッシュボードで同時確認する

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [on_call_rotation_handover](01_on_call_rotation_handover.md)
- [SRE_50pc_rule_toil削減](08_SRE_50pc_rule_toil削減.md)
