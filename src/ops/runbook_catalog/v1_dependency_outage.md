---
id: ops.runbook.v1_dependency_outage
signal_class: v1_dependency_outage
runbook_class: v1_full
drill_cadence_days: 60
rto_minutes: 90
status: active
---

# runbook: v1_dependency_outage — 外部依存障害

## 概要

上位/下位 service / 外部 SaaS / IDP / DNS のいずれかが利用不可になった場合のシグナル runbook。

## phase_1_detect（page → ack ≤ 5 min）

- Alertmanager が `k1s0-dependency-outage` を発火する。
- Prometheus probe が連続 3 回失敗した時点で発火する。

## phase_2_triage（ack → triage ≤ 30 min）

1. 影響範囲を特定する: IDP / DNS / upstream service / downstream consumer
2. IDP 障害の場合: Keycloak / OIDC discovery endpoint の状態確認
3. DNS 障害: `dig @<nameserver> <fqdn>` で解決可否を確認する

## phase_3_mitigate（≤ 15 min）

### IDP 障害

1. OIDC fallback mode を有効化する（BFF auth edge の feature flag を切り替える）:
   ```sh
   # flagd の ConfigMap を更新して IDP failsafe mode を有効化する
   kubectl patch configmap k1s0-flagd-config -n k1s0-system \
     --patch '{"data":{"idp_failsafe_mode":"true"}}'
   ```

### 外部 SaaS 障害

1. circuit breaker を open にして cascading failure を防ぐ。
2. degraded mode のフォールバックレスポンスを有効化する。

## phase_4_resolve

- 依存サービスが復旧して 3 分間連続成功を確認する。
- failsafe / circuit breaker を通常状態に戻す。

## phase_5_postmortem

- `docs/postmortem/YYYY-MM-DD_dependency_outage.md` に起票する。
- vendor SLA レビューを action item に含める。
