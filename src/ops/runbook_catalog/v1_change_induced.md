---
id: ops.runbook.v1_change_induced
signal_class: v1_change_induced
runbook_class: v1_full
drill_cadence_days: 30
rto_minutes: 30
status: active
---

# runbook: v1_change_induced — 変更誘発 SLO 急変

## 概要

deploy / migration / canary / topology 変更直後の SLO 急変シグナルの runbook。auto rollback が trigger される場合は freeze を併用する。

## phase_1_detect（page → ack ≤ 5 min）

- Argo Rollouts AnalysisTemplate が burn rate 超過を検知して自動 pause / rollback を開始する。
- Alertmanager が `k1s0-change-induced-critical` を発火する。

## phase_2_triage（ack → triage ≤ 30 min）

1. 直近の deploy / migration を確認する: `kubectl rollout history deployment/<name>`
2. Argo Rollouts の状態確認: `kubectl argo rollouts get rollout <name> -n k1s0-system --watch`
3. 変更種別を特定する（canary / migration / topology）

## phase_3_mitigate（freeze 時 ≤ 5 min）

### auto rollback が動作している場合

```sh
# Argo Rollouts の auto rollback 状態を確認する
kubectl argo rollouts get rollout <name> -n k1s0-system
```

### 手動 rollback が必要な場合

```sh
# 直前の安定リビジョンに rollback する
kubectl rollout undo deployment/<name> -n k1s0-system
# freeze を発動する
kubectl patch configmap k1s0-slo-status -n k1s0-system \
  --patch '{"data":{"error_budget_freeze":"true"}}'
```

## phase_4_resolve

- rollback 完了後に SLO burn rate が 1x 以下に戻るまで 15 分監視する。
- freeze 解除は dual sign-off 後: `error_budget_freeze: false`

## phase_5_postmortem

- `docs/postmortem/YYYY-MM-DD_change_induced.md` に起票する。
- action item: 変更手順の gate check 強化 / AnalysisTemplate の閾値調整。
