---
id: ops.runbook.v1_capacity_breach
signal_class: v1_capacity_breach
runbook_class: v1_full
drill_cadence_days: 60
rto_minutes: 120
status: active
---

# runbook: v1_capacity_breach — 容量上限超過

## 概要

per_tenant_volume / cluster headroom / partition capacity のいずれかが hard threshold を超過した場合のシグナル runbook。

## phase_1_detect（page → ack ≤ 5 min）

- Alertmanager が `k1s0-capacity-warning` または `k1s0-capacity-critical` を発火する。
- Tekton Trigger が本 runbook を自動起動する。

## phase_2_triage（ack → triage ≤ 30 min）

1. `kubectl top nodes` でクラスタ headroom を確認する。
2. テナント quota を確認する: `kubectl get resourcequota -n <tenant-ns>`.
3. Kafka partition の lag を確認する: `kafka-consumer-groups.sh --bootstrap-server ... --describe --all-groups`.

## phase_3_mitigate（≤ 15 min）

### テナント容量超過

```sh
# テナント quota を一時的に拡張する (dual sign-off 後に実施)
kubectl patch resourcequota <tenant>-quota -n <tenant-ns> \
  --patch '{"spec":{"hard":{"requests.cpu":"4","requests.memory":"8Gi"}}}'
```

### クラスタ headroom 不足

1. HPA を手動スケールアウトする: `kubectl scale deployment <name> --replicas=<n>`
2. ノード自動スケールの確認: `kubectl get nodes`

## phase_4_resolve

- 容量が threshold の 80% 以下に戻るまで監視する。
- 恒久対処として tier2/quota の classes.yaml を更新する PR を起票する。

## phase_5_postmortem

- `docs/postmortem/YYYY-MM-DD_capacity_breach.md` に起票する。
