---
id: plan.overview.scenario_ops_incident_response_7class
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

# incident_response_7class主導

## 一文方針

7 incident class (secret_exposure / data_exfiltration / data_tampering / availability_loss / supply_chain / privilege_escalation / audit_chain_break) の 6 phase (検知 → triage → contain → eradicate → recover → postmortem) を主導し、全関与者の coordination を完了させる。

> 水曜午後 3 時、Mattermost `#security-incident` に「本番テナント A のデータ exfiltration が疑われる」という alert が届く。ops 担当者が IR commander として自動 assign される。`scenario_catalog.lock.yaml` の `v1_data_exfiltration` playbook を開き、triage phase で影響テナントと exfiltrated データ種別を特定する。security 担当者・data 担当者・infra 担当者を招集し、6 phase の coordination を開始する。

## ペルソナ要約

主役: ops 担当者（シニア級、IR commander）、目的: 7 incident class の 6 phase playbook を主導し、全関与者の coordination で incident を完結させる

## 現状業務での痛み

- incident 発生時に誰が IR commander になるかが曖昧で、初動対応が遅延する
- 7 incident class ごとの対応手順が存在せず、担当者が毎回一から考える
- 複数担当者の coordination が口頭に依存し、フォローアップが漏れる
- postmortem までを incident の一部として捉えておらず、improve phase が形骸化している

## k1s0 でこう変わる

- `escalation_policy.lock.yaml` による IR commander の自動 assign で初動対応遅延がなくなる
- `scenario_catalog.lock.yaml` の 7 incident class × 6 phase playbook で対応手順が標準化される
- Mattermost incident チャンネルが自動作成され、全関与者の coordination 履歴が記録される
- postmortem phase まで incident として追跡され、release_gate との連動で改善が担保される

## Trigger

incident が検知された時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 週次〜月次（incident 発生時）
- 典型きっかけ: 「Mattermost `#security-incident` に data_exfiltration の疑いが届いた」「SLO burn rate が 14.4x を超え availability_loss incident が宣言された」
- 頻度根拠: incident class 別に発生頻度が異なる。availability_loss が最多（月次）、secret_exposure / privilege_escalation は年次〜不定期

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役、IR commander） | シニア | 本社 IT 室 / リモート | Mattermost #ops-incident / scenario_catalog.lock.yaml | 6 phase 主導・全関与者 coordination |
| security 担当者 | シニア | 本社 IT 室 / リモート | Mattermost #security-incident | contain / eradicate phase の security アクション |
| data 担当者 | シニア | 本社 IT 室 / リモート | Mattermost #ops-incident | data_exfiltration / data_tampering の data 層対応 |
| infra 担当者 | シニア | 本社 IT 室 / リモート | Mattermost #ops-incident | availability_loss / supply_chain の infra 層対応 |
| tier1 担当者 | シニア | 本社 IT 室 / リモート | Mattermost #ops-incident | supply_chain / privilege_escalation の tier1 層対応 |

## 個人 KPI / 達成感

- incident 宣言から contain phase 完了まで: P1 は 1 時間以内 / P2 は 4 時間以内
- postmortem PR 作成: recover 完了から 48 時間以内（シナリオ 03 と連動）
- SLO error budget 消費: 各 incident class の SLA 以内に回収

## 工数 / 関与人数 / コスト感

- P1 incident（availability_loss 等）: 2〜8 時間
- P2 incident（data_exfiltration 調査等）: 4〜24 時間
- 関与人数: 2〜5 名（IR commander + 各軸担当者）

## 前提

- `scenario_catalog.lock.yaml` に 7 incident class × 6 phase playbook が定義されている
- `escalation_policy.lock.yaml` に IR commander の自動 assign が設定されている
- Mattermost に incident チャンネル自動作成機能が設定されている

## 流れ

1. **検知 (phase 1)**: Mattermost `#security-incident` または `#ops-incident` で incident class を確認し、IR commander として自動 assign を受ける
2. **triage (phase 2)**: `scenario_catalog.lock.yaml` の対応 incident class playbook を開く。影響範囲・severity・影響テナントを特定し、P1 / P2 / P3 を判定する。Mattermost incident チャンネルを作成し、関与者を招集する
3. **contain (phase 3)**: incident class 別の contain アクションを実行する（例: data_exfiltration → PII cluster の write 停止 / availability_loss → traffic shed / supply_chain → Harbor quarantine）
4. **eradicate (phase 4)**: 根本原因を特定し除去する（security 担当者・data 担当者・infra 担当者と協働）
5. **recover (phase 5)**: サービスを正常状態に回復し、SLO dashboard で全 signal_class の正常復帰を確認する
6. **postmortem (phase 6)**: postmortem phase 到達を Mattermost に通知し、postmortem PR 作成（シナリオ 03）を開始する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | alert system | incident class 検知・Mattermost 通知 | 「[incident] data_exfiltration 疑い: テナント A」 |
| T+5m | ops 担当者 | IR commander assign・triage 開始 | Mattermost: 「#incident-20260512-001 チャンネル作成」 |
| T+15m | ops 担当者 | 影響範囲・severity 確定・関与者招集 | Mattermost: 「全関与者招集: security / data / infra」 |
| T+30m | 全関与者 | contain phase 開始 | — |
| T+60m | 全関与者 | eradicate phase 開始 | — |
| T+120m | 全関与者 | recover phase・SLO 復帰確認 | 「[incident] サービス正常復帰を確認」 |
| T+125m | ops 担当者 | postmortem phase 通知 | Mattermost: 「postmortem phase 開始。シナリオ 03 を開始」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | availability_loss incident は警報配信 SLA に直結 |
| FA 生産指示 | 高 | data_tampering / availability_loss は生産指示の整合性に直結 |
| SCADA テレメトリ収集 | 高 | data_exfiltration は SCADA データの機密性に直結 |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- security 強制機構
- 関連 OSS: Mattermost（incident チャンネル）/ GitHub（incident tracker）/ Backstage（playbook）

## 期待結果 / 観測指標

- P1 incident の contain 完了が 1 時間以内
- 7 incident class 全て `scenario_catalog.lock.yaml` に playbook が存在する
- postmortem PR が recover 完了後 48 時間以内に作成されている

## 失敗時の挙動 / escalation

- **IR commander が応答しない（10 分以内）**: `escalation_policy.lock.yaml` の L3 escalation が発火し、tech lead と ops チームリーダーに通知する
- **playbook に対応手順が存在しない（未知 incident class）**: ops 担当者が既存の最近似 incident class の playbook を準拠しつつ対応し、postmortem で新 incident class の playbook 追加を action item に登録する
- **contain phase が 1 時間以内に完了しない（P1）**: L3 escalation を発火し、経営判断（DR 宣言等）を求める

## 失敗パターン (anti-pattern)

1. **triage を省略して contain に直行する**: 影響範囲が不明なまま contain を実施するため、over-containment または under-containment が発生する
2. **IR commander が全作業を自分でやろうとする**: coordination が属人化し、対応速度が落ちる。IR commander は主導・判断・报告に専念し、実作業は各軸担当者に委任する
3. **recover 確認なしに incident を close する**: SLO が未復帰のまま close され、次の shift で再発に気づくケースがある

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [postmortem_PR_merge_gate](03_postmortem_PR_merge_gate.md)
- [SLO監視_burn_rate_alert](02_SLO監視_burn_rate_alert.md)
- [security: data_breach_privacy_incident_response](../../06_security担当者シナリオ/14_data_breach_privacy_incident_response.md)
