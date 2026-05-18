---
id: ops.runbook.v1_slo_breach
signal_class: v1_slo_breach
runbook_class: v1_full
drill_cadence_days: 30
rto_minutes: 60
status: active
---

# runbook: v1_slo_breach — SLO error budget burn rate 超過

## 概要

13 SLO のいずれかの error budget burn rate が 6x を超過した場合に発火するシグナルの runbook。
14.4x 以上の場合は `error_budget_freeze` を `true` に設定して全 deploy を block する。

## phase_1_detect（page → ack ≤ 5 min）

- Alertmanager が `k1s0-slo-critical` alert を発火する。
- PagerDuty / Mattermost でオンコールに通知する。
- Tekton Trigger が本 runbook を自動起動する。

## phase_2_triage（ack → triage ≤ 30 min）

1. `kubectl get slo -n k1s0-system` で burn rate を確認する。
2. Perses ダッシュボード `ops_loop` で影響 SLO を特定する。
3. 直近の deploy / migration / topology 変更を確認する（変更誘発なら `v1_change_induced` に昇格）。
4. IR ticket を Mattermost `#incident` チャンネルに起票する。

## phase_3_mitigate（page 時 ≤ 15 min、freeze 時 ≤ 5 min）

### 14.4x 以上（freeze 発動）

```sh
# error_budget_freeze を true に設定して全 deploy を block する
kubectl patch configmap k1s0-slo-status -n k1s0-system \
  --patch '{"data":{"error_budget_freeze":"true"}}'
```

### 6x ～ 14.4x（通常 page）

1. Argo Rollouts で直近 canary を一時停止する: `kubectl argo rollouts pause <name> -n k1s0-system`
2. Envoy rate limit を tier1 SLO config に従い絞る（`/admin/v1/rate_limit/override`）。
3. 原因 service の pod を再起動して OOM / memory leak を解消する。

## phase_4_resolve（SLO budget 復活まで）

- burn rate が 1x 以下に戻るまで監視する（最低 10 分維持）。
- freeze を発動していた場合は dual sign-off 後に解除する:

```sh
kubectl patch configmap k1s0-slo-status -n k1s0-system \
  --patch '{"data":{"error_budget_freeze":"false"}}'
```

## phase_5_postmortem（open ≤ 72h / merged ≤ 14d）

- `docs/postmortem/YYYY-MM-DD_slo_breach_<service>.md` に blameless postmortem を起票する。
- 必須 6 section: timeline / root_cause / impact / action_items / lessons_learned / preventions。
- action item を feature branch に分岐させて PR を起票する。
