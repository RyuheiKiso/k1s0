---
runbook_id: RB-API-001
title: tier1 API レイテンシ劣化対応
category: API
severity: SEV2〜SEV3
owner: 起案者
automation: manual
alertmanager_rule: TierOneLatencyHigh
fmea_id: 間接対応
estimated_recovery: 暫定 30 分 / 恒久 24 時間
last_updated: 2026-05-02
---

# RB-API-001: tier1 API レイテンシ劣化対応

本 Runbook は tier1 facade の p99 レイテンシが 500ms を 5 分継続して超過した時の対応を定める。
SLA 99% / NFR-A-SLA-001 / NFR-B-PERF-001 に対応する。継続時間と影響範囲で SEV2 / SEV3 を判定。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + Argo CD app 権限を保持。
- 必要ツール: `kubectl` / `argocd` / Grafana / Tempo / Loki アクセス。
- kubectl context が `k1s0-prod`。
- tier1 facade は namespace `k1s0`、Deployment 名 `tier1-facade`。

## 2. 対象事象

- Alertmanager `TierOneLatencyHigh` 発火（`histogram_quantile(0.99, sum(rate(grpc_server_handling_seconds_bucket{namespace="k1s0"}[5m])) by (le)) > 0.5`）、または
- 利用者からの問合せで API の遅延報告。

検知シグナル:

```promql
# tier1 facade gRPC p99 レイテンシ
histogram_quantile(0.99,
  sum(rate(grpc_server_handling_seconds_bucket{namespace="k1s0"}[5m])) by (le)) > 0.5

# 個別コンポーネント別の遅延
histogram_quantile(0.99,
  sum(rate(grpc_server_handling_seconds_bucket{service=~"State|Secrets|PubSub"}[5m]))
  by (service, le))

# 依存先（DB / Valkey / Kafka）の遅延
pg_stat_activity_avg_query_duration_ms > 100
valkey_commands_duration_seconds_p99 > 0.05
```

ダッシュボード: **Grafana → k1s0 tier1 SLO**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#alert-tier1`。

## 3. 初動手順（5 分以内）

```bash
# tier1 facade Pod の状態
kubectl get pods -n k1s0 -l app=tier1-facade -o wide

# Pod のリソース消費
kubectl top pod -n k1s0 -l app=tier1-facade

# 直近 5 分のエラーログ
kubectl logs -n k1s0 -l app=tier1-facade --tail=100 --since=5m | grep -iE "ERROR|FATAL|timeout"
```

```bash
# 依存コンポーネント別の遅延を Grafana で確認
# Grafana → k1s0 tier1 SLO → "by service breakdown" パネル
```

ステークホルダー通知: Severity 判定（[`RB-INC-001`](RB-INC-001-severity-decision-tree.md) 参照）後、Slack `#alert-tier1` に通知。
SEV1 昇格条件: エラーバジェット消費率 > 20%/h、または複数 tier 影響。

## 4. 原因特定手順

```bash
# Tempo で遅いトレースを特定
# Grafana → Explore → Tempo → service.name="tier1-facade" duration > 500ms

# Loki で関連ログ
logcli query '{namespace="k1s0", app="tier1-facade"} | json
  | duration_ms > 500' --since=15m | head -20
```

よくある原因:

1. **DB 過負荷**: PostgreSQL connection pool 枯渇 / slow query。`pg_stat_statements` で重いクエリを特定。[`RB-DB-002`](RB-DB-002-postgres-primary-failover.md) と連鎖。
2. **Valkey 遅延**: Cluster ノード障害 / メモリ枯渇。[`RB-DB-001`](RB-DB-001-valkey-node-failover.md) と連鎖。
3. **Kafka 遅延**: broker 過負荷 / partition 偏り。[`RB-MSG-001`](RB-MSG-001-kafka-broker-failover.md) と連鎖。
4. **Pod CPU 飽和**: HPA 反応が遅く Pod 不足。`kubectl top pod` で確認、scale up。
5. **mTLS handshake 遅延**: SPIRE SVID 取得失敗。[`RB-SEC-002`](RB-SEC-002-cert-expiry.md) と連鎖。
6. **直前のデプロイ起因**: コードバグでパフォーマンス劣化。Argo CD 履歴を確認。

## 5. 復旧手順

### 暫定対応: スケールアウト（〜10 分）

```bash
# tier1-facade を手動 scale up
kubectl scale deployment/tier1-facade -n k1s0 --replicas=10
kubectl rollout status deployment/tier1-facade -n k1s0
```

### 直前デプロイ起因なら rollback

```bash
ops/scripts/rollback.sh \
  --app k1s0-tier1-facade \
  --revision <prev-good-sha> \
  --reason "RB-API-001: latency degradation"
```

### 依存先障害なら該当 Runbook を起動

- DB 過負荷 → [`RB-DB-002`](RB-DB-002-postgres-primary-failover.md)
- Valkey 遅延 → [`RB-DB-001`](RB-DB-001-valkey-node-failover.md)
- Kafka 遅延 → [`RB-MSG-001`](RB-MSG-001-kafka-broker-failover.md)

### 恒久対応

- slow query の index 追加 PR
- Pod resource limits の見直し
- HPA 閾値の調整（CPU 70% → 50% に下げる等）

## 6. 検証手順

復旧完了の判定基準:

- p99 レイテンシが 500ms 未満を 15 分継続。
- エラーレートが 1% 未満。
- 直近 30 分の Loki クエリ `{app="tier1-facade"} |= "timeout"` が 0 件。
- 主要 API（State.Get / Secrets.Get 等）の合成監視で OK。

## 7. 予防策

- ポストモーテム起票（72 時間以内、`postmortems/<YYYY-MM-DD>-RB-API-001.md`）。
- HPA / VPA の閾値見直し。
- slow query の継続監視（Top 10 を週次レビュー）。
- 月次 Chaos Drill 対象に「DB 接続レイテンシ +200ms」シナリオを追加。
- NFR-A-REC-002 / NFR-B-PERF-001 の MTTR / SLO ログを更新。

## 8. 関連 Runbook

- 関連 NFR: [NFR-A-SLA-001 / NFR-B-PERF-001](../../../docs/03_要件定義/30_非機能要件/B_性能.md)
- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/07_負荷試験方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/07_負荷試験方式.md)
- 連鎖 Runbook:
  - [`RB-DB-002-postgres-primary-failover.md`](RB-DB-002-postgres-primary-failover.md)
  - [`RB-DB-001-valkey-node-failover.md`](RB-DB-001-valkey-node-failover.md)
  - [`RB-MSG-001-kafka-broker-failover.md`](RB-MSG-001-kafka-broker-failover.md)
  - [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md)
- 関連 load test: [`../../load/scenarios/state_baseline.js`](../../load/scenarios/state_baseline.js)
