---
id: ops.runbook.v1_drill_failure
signal_class: v1_drill_failure
runbook_class: v1_lightweight
drill_cadence_days: 90
rto_minutes: 480
status: active
---

# runbook: v1_drill_failure — ドリル想定外 failure

## 概要

chaos drill / restore drill / red team drill で想定外 failure mode に到達した場合のシグナル runbook。緊急度は低いが、次のドリルまでに対処する。

## phase_1_detect

- Litmus chaos experiment / Tekton dry-run pipeline が非ゼロ exit code で終了した場合に発火する。
- Alertmanager が `k1s0-drill-failure` を発火する（severity: warning）。

## phase_2_triage（≤ 30 min）

1. ドリルの種別と失敗内容を確認する。
2. failure mode が想定外かどうかを判断する（既知の制限なら `accepted_with_assumption`）。
3. SRE Slack チャンネル `#sre-drills` に報告する。

## phase_3_mitigate（≤ 8h）

1. ドリル環境をリストアして production と同等状態に戻す。
2. 失敗原因を文書化する。
3. 修正 PR を起票する（緊急度に応じて sprint に追加）。

## phase_4_resolve

- 修正後にドリルを再実施して成功を確認する。

## phase_5_postmortem

- `docs/postmortem/YYYY-MM-DD_drill_failure_<drill_type>.md` に軽量 postmortem を起票する。
- action item: drill scenario の更新 / 想定外 failure の恒久対処。
