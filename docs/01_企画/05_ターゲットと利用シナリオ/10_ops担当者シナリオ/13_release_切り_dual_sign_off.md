---
id: plan.overview.scenario_ops_release_dual_sign_off
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

# release_切り_dual_sign_off

## 一文方針

release 切りに際して postmortem PR merge の確認 / SLO error budget 確認 / `release_gate.lock.yaml` 全 cell green 確認を経て、ops + 関係軸担当者の dual sign-off を完了させる。

> 四半期末の金曜、ops 担当者が `release_gate.lock.yaml` の全 cell を確認する。postmortem_pending: green / cve_overdue_count: 0 / error_budget_remaining: 28% / capacity_planning_completed: green を確認し、tier1 担当者と security 担当者に dual sign-off を依頼する。全 sign-off が揃ったことを確認し、Argo CD で production release をトリガする。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: `release_gate.lock.yaml` 全 cell green と dual sign-off を完了させ、release を安全に実施する

## 現状業務での痛み

- release 判断が担当者の「勘」に依存しており、CVE 未対応や postmortem 未完了のまま release が走るケースがある
- dual sign-off の手順が明文化されておらず、誰に何を確認してもらうべきかが曖昧
- SLO error budget が枯渇寸前でも release を強行し、次の incident で error budget が完全に尽きるケースがある
- release gate のチェック項目が散在し、抜け漏れが発生する

## k1s0 でこう変わる

- `release_gate.lock.yaml` が release の物理 prerequisite として機能し、全 cell green でなければ release が技術的に不可能になる
- dual sign-off が構造化され、ops + 関係軸担当者の sign-off 記録が GitHub commit として残る
- SLO error budget の残余が release gate の必須チェック項目となり、error budget 枯渇での release が防止される
- release gate チェック項目が単一ファイルに集約され、抜け漏れがなくなる

## Trigger

release milestone 達成時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次〜四半期
- 典型きっかけ: 「v1.8.0 の開発が完了し、release milestone が達成された」「四半期 release cadence のタイミングが到来した」
- 頻度根拠: release cadence に依存。月次 release または四半期 release が典型

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | release_gate.lock.yaml / Grafana SLO dashboard | gate 確認・dual sign-off 依頼・release trigger |
| tier1 担当者（sign-off） | シニア | 本社 IT 室 / リモート | release_gate.lock.yaml | supply chain / CVE cell 確認・sign-off |
| security 担当者（sign-off） | シニア | 本社 IT 室 / リモート | release_gate.lock.yaml | security cell 確認・sign-off |

## 個人 KPI / 達成感

- `release_gate.lock.yaml` 全 cell green での release 率: 100%
- dual sign-off 取得時間: release gate 確認から 24 時間以内
- release 後 24 時間以内の rollback 発生率: 目標 5% 以下

## 工数 / 関与人数 / コスト感

- release gate 確認: 30〜60 分
- dual sign-off 依頼 + 取得: 1〜4 時間
- release trigger + progressive delivery 立会（シナリオ 05）: 2〜4 時間
- 関与人数: 3〜5 名（ops + sign-off 担当者 + 必要軸担当者）

## 前提

- `release_gate.lock.yaml` が build artifact として存在し、全 cell が自動更新される
- dual sign-off の sign-off 者が `escalation_policy.lock.yaml` で定義されている
- Argo CD が release gate 確認後の production rollout を自動開始する設定になっている

## 流れ

1. **release milestone 確認**: release milestone が達成されたことを GitHub Milestone で確認する
2. **`release_gate.lock.yaml` 確認**: 全 cell を確認する
   - `postmortem_pending`: green（未完了 postmortem が 0 件）
   - `cve_overdue_count`: 0（patch SLO 超過 CVE が 0 件）
   - `error_budget_remaining`: 15% 以上
   - `capacity_planning_completed`: green
   - `build_determinism_failure`: 0
3. **SLO error budget 確認**: Grafana SLO dashboard で全 SLO の error budget 残余を確認する
4. **red cell 対応**: red cell がある場合は対応担当者に escalation し、green になるまで release を保留する
5. **dual sign-off 依頼**: tier1 担当者（supply chain / CVE cell 担当）と security 担当者（security cell 担当）に sign-off を依頼する
6. **sign-off 記録確認**: 全 sign-off が `release_gate.lock.yaml` に GitHub commit として記録されたことを確認する
7. **release trigger**: Argo CD で production の release rollout を開始する（シナリオ 05 の Argo Rollouts progressive delivery に移行）
8. **Mattermost 報告**: Mattermost `#ops-releases` に release 開始を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | ops 担当者 | release_gate.lock.yaml 全 cell 確認 | — |
| T+1h | ops 担当者 | SLO error budget 確認完了 | — |
| T+2h | ops 担当者 | dual sign-off 依頼（tier1 / security） | Mattermost DM: 「v1.8.0 release sign-off をお願いします」 |
| T+6h | tier1 担当者 | supply chain / CVE cell 確認・sign-off | GitHub: commit (tier1 sign-off) |
| T+8h | security 担当者 | security cell 確認・sign-off | GitHub: commit (security sign-off) |
| T+9h | ops 担当者 | 全 sign-off 確認・release trigger | Mattermost: 「v1.8.0 release 開始」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信コンポーネントの release は全業務担当者に影響 |
| FA 生産指示 | 高 | 生産指示配信の release は製造 SLA に直結 |
| 全 9 業務 | 高 | release gate は全業務を対象とした横断 prerequisite |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- 検証規律適合仕様
- 関連 OSS: Argo CD（release trigger）/ GitHub（sign-off 記録）/ Grafana（SLO dashboard）/ Mattermost（通知）

## 期待結果 / 観測指標

- `release_gate.lock.yaml` 全 cell green での release が 100%
- dual sign-off（tier1 + security）が GitHub commit として記録されている
- release 後 24 時間以内の rollback 発生率が 5% 以下

## 失敗時の挙動 / escalation

- **`cve_overdue_count` が 0 にならない**: tier1 担当者に CVE patch を escalation し、patch 完了まで release を保留する。patch SLO 超過時は L3 escalation
- **`error_budget_remaining` が 15% を下回る**: 次の release cadence まで release を延期する。緊急 release が必要な場合は tech lead と経営判断を経て例外処理を実施する
- **sign-off 担当者が 24 時間以内に応答しない**: `escalation_policy.lock.yaml` の L2 escalation が発火し、チームリーダーが代替 sign-off 者を assign する

## 失敗パターン (anti-pattern)

1. **`release_gate.lock.yaml` を確認せずに release trigger する**: postmortem 未完了や CVE 未対応のまま release が走り、セキュリティリスクと品質問題が顕在化する
2. **dual sign-off を省略して ops 単独で release trigger する**: 構造的に dual sign-off なしの release は不可だが、手順をバイパスしようとしない
3. **error budget が 5% を下回っても「今月は特別」と例外処理する**: error budget 枯渇後の次 incident で即座に SLA 違反になる

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [Argo_Rollouts_progressive_delivery](05_Argo_Rollouts_progressive_delivery.md)
- [postmortem_PR_merge_gate](03_postmortem_PR_merge_gate.md)
