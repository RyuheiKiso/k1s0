---
runbook_id: RB-MSG-002
title: Dead Letter Queue 滞留対応
category: MSG
severity: SEV2
owner: 協力者
automation: manual
alertmanager_rule: KafkaDLQBacklogHigh
fmea_id: 間接対応（FMEA-003 連鎖）
estimated_recovery: 暫定 15 分 / 恒久 1 時間
last_updated: 2026-05-02
---

# RB-MSG-002: Dead Letter Queue 滞留対応

本 Runbook は Kafka DLQ（`*.dlq` トピック）にメッセージが滞留した時の対応を定める。
DLQ 滞留は下流コンシューマの不安定性を示唆し、データ整合性リスクを伴う。NFR-C-MON-002 / DS-OPS-RB-011 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + `kafka` namespace の Pod exec / log 権限を保持。
- 必要ツール: `kubectl` / `kafka-topics.sh` / `kafka-mirror-maker.sh` / `kafka-consumer-groups.sh`（broker Pod 内で実行）/ `logcli` / Tempo 検索。
- kubectl context が `k1s0-prod`。
- Kafka クラスタ自体が healthy であること（broker 障害時は [`RB-MSG-001`](RB-MSG-001-kafka-broker-failover.md) 先行）。
- Dapr pubsub component（`infra/dapr/components/pubsub/kafka.yaml`）が configure 済み。

## 2. 対象事象

- Alertmanager `KafkaDLQBacklogHigh` 発火（`kafka_log_log_size{topic=~".*\\.dlq"} > 500` を 30 分継続）、または
- DLQ Consumer の lag 増加（`kafka_consumergroup_lag{topic=~".*\\.dlq"} > 0`）、または
- DLQ プロデュース率上昇（`rate(kafka_server_brokertopicmetrics_messagesinpersec{topic=~".*\\.dlq"}[5m]) > 1`）。

検知シグナル:

```promql
# DLQ トピックのメッセージ積算数（30 分で 500 件超でアラート）
kafka_log_log_size{namespace="kafka", topic=~".*\\.dlq"} > 500

# consumer group の lag（DLQ consumer が止まっている場合）
kafka_consumergroup_lag{namespace="kafka", topic=~".*\\.dlq"} > 0

# DLQ へのプロデュース率（上流でエラーが多発している指標）
rate(kafka_server_brokertopicmetrics_messagesinpersec{topic=~".*\\.dlq"}[5m]) > 1
```

ダッシュボード: **Grafana → k1s0 DLQ Overview**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#incident-kafka`。

## 3. 初動手順（5 分以内）

```bash
# どの DLQ トピックが滞留しているか特定
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-topics.sh --bootstrap-server localhost:9093 --list | grep dlq
```

```bash
# 各 DLQ のメッセージ数を確認
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-log-dirs.sh --bootstrap-server localhost:9093 \
  --topic-list <dlq-topic> --describe | grep -i size
```

```bash
# DLQ メッセージのサンプルを取得して失敗原因を確認
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-console-consumer.sh \
  --bootstrap-server localhost:9093 \
  --topic <dlq-topic> \
  --from-beginning --max-messages 5
```

```bash
# 上流 tier1 facade の error ログを確認
kubectl logs -n k1s0 deploy/tier1-facade --tail=100 | grep -i "dlq\|dead.letter\|retry"
```

```bash
# Dapr pubsub subscription の失敗ステータスを確認
kubectl get components -n k1s0 | grep kafka
kubectl describe subscription -n k1s0
```

ステークホルダー通知: Slack `#incident-kafka` に「DLQ <topic> に <N> 件滞留、原因調査中」を 5 分以内に投稿。
SEV2 のため `oncall/escalation.md` 起動は不要だが、根本原因が PII 流出経路を含む場合は SEV1 昇格して即時起動。

## 4. 原因特定手順

```logql
# Loki で DLQ 関連ログ
{namespace="k1s0", app="tier1-facade"} |= "dlq" | json | line_format "{{.msg}}"
```

Tempo で trace を確認:

```bash
# DLQ 転送時に付与された trace-id で Tempo を検索
# Grafana → Explore → Tempo → TraceID: <id>
```

よくある原因:

1. **コンシューマのバグ（例外がキャッチされず retryable 扱い）**: 特定のメッセージ形式でパニックが発生し、全メッセージが DLQ に流れる。スタックトレースを `kubectl logs` で確認。修正は新版デプロイ。
2. **依存サービス障害（DB タイムアウト / SPIRE SVID 失効）**: コンシューマ処理中に接続失敗 → 全メッセージが retry 上限に達して DLQ 転送。依存先障害は [`RB-DB-002`](RB-DB-002-postgres-primary-failover.md) / [`RB-SEC-002`](RB-SEC-002-cert-expiry.md) を並行起動。
3. **スキーマ進化の非互換**: Proto / Avro のフィールド削除でデシリアライズ失敗。`UnknownFieldSetHandler` のログを確認。スキーマ進化が原因なら schema-registry の互換性チェック設定見直し。
4. **max.poll.interval.ms 超過**: 処理が遅く consumer が group から kick される。Rebalance ループで DLQ 転送が増加する。`max.poll.interval.ms` を引き上げ + consumer 並列度を上げる。
5. **メッセージサイズ超過（1 MB デフォルト）**: `max.message.bytes` 設定と送信ペイロードのサイズを確認。tier1 の `FR-T1-PUBSUB-005`（1 MiB 上限）が機能しているか確認。

## 5. 復旧手順

### Step 1 — 根本原因を修正する

- コンシューマのコードバグ: 修正版をデプロイしてから DLQ をリプレイする。
- 依存サービスの一時障害: 対象サービスが復旧済みであることを確認する（PostgreSQL / SPIRE 等）。
- スキーマ不整合: Avro スキーマ変更によるデシリアライズ失敗の場合はスキーマバージョンを合わせる。

### Step 2 — DLQ メッセージを本来のトピックに再投入する

```bash
# DLQ から本来のトピックへ転送（Kafka native mirror）
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-mirror-maker.sh \
  --consumer.config /tmp/consumer.properties \
  --producer.config /tmp/producer.properties \
  --whitelist "<dlq-topic>" \
  --num.streams 1
```

または Dapr コンポーネントを使って tier1 facade 経由でリプレイするエンドポイントを呼ぶ（実装済みの場合）。

### Step 3 — リプレイ完了後、DLQ のオフセットをリセットする

```bash
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-consumer-groups.sh \
  --bootstrap-server localhost:9093 \
  --group <consumer-group> \
  --reset-offsets --to-latest \
  --topic <dlq-topic> --execute
```

リプレイ進捗のモニタリング:

```promql
kafka_consumergroup_lag{namespace="kafka", topic=~".*\\.dlq"} == 0
```

## 6. 検証手順

復旧完了の判定基準:

- 全 DLQ トピックの `kafka_log_log_size` が 5 件未満まで減少（リプレイ完了）。
- `kafka_consumergroup_lag{topic=~".*\\.dlq"}` == 0 が 15 分間継続。
- DLQ プロデュース率 `rate(...messagesinpersec{topic=~".*\\.dlq"}[5m])` < 0.1（新規 DLQ 流入が止まった）。
- 上流 tier1 facade の `/healthz` が 200、直近 10 分の Loki クエリ `{namespace="k1s0"} |= "dead.letter"` が 0 件。
- リプレイ後の本来トピックで処理エラーが発生していない（Loki `{app="tier1-facade"} |= "ERROR"` で確認）。
- データ整合性: リプレイ後にダウンストリームの集計値（例: 監査レコード数）が想定範囲内。

## 7. 予防策

- ポストモーテム起票（72 時間以内、`postmortems/<YYYY-MM-DD>-RB-MSG-002.md`）。
- DLQ アラートの閾値（500 件 / 30 分）が適切か見直す。連続滞留パターンに合わせて調整。
- DLQ リプレイ自動化 Runbook タスクを検討（[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「自動化対象の識別」、月 3 回以上で対象）。
- コンシューマの retry 設定（`max.retries` / `backoff.ms`）を見直す。指数バックオフ + Circuit Breaker を組合せ。
- NFR-A-REC-002 の MTTR ログを更新（目標: 暫定 15 分 / 恒久 1 時間）。
- 月次 Chaos Drill 対象に「DLQ 1000 件挿入」シナリオを追加（`ops/chaos/experiments/dlq-flood.yaml` 予定）。

## 8. 関連 Runbook

- 関連設計書: `infra/data/kafka/kafka-cluster.yaml`、`infra/dapr/components/pubsub/kafka.yaml`、[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §DS-OPS-RB-011
- 関連 ADR: [ADR-DATA-002（Kafka）](../../../docs/02_構想設計/adr/ADR-DATA-002-kafka.md)
- 関連 NFR: [NFR-C-MON-002](../../../docs/03_要件定義/30_非機能要件/C_運用.md), [NFR-A-REC-002](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 FMEA: 間接対応（FMEA-003 連鎖）
- 連鎖 Runbook:
  - [`RB-MSG-001-kafka-broker-failover.md`](RB-MSG-001-kafka-broker-failover.md) — Kafka broker 障害が DLQ 流入の根本原因の場合
  - [`RB-DB-002-postgres-primary-failover.md`](RB-DB-002-postgres-primary-failover.md) — DB タイムアウトが DLQ 起因の場合
  - [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) — SPIRE SVID 失効が DLQ 起因の場合
