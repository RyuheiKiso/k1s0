# Dead Letter Queue 滞留 Runbook

> **alert_id**: tier1.kafka.dlq.backlog-high
> **severity**: SEV2
> **owner**: tier1-platform-team
> **estimated_mttr**: 45m
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

**Mimir / Grafana** で以下を確認する。

PromQL（Mimir）:

```promql
# DLQ トピックのメッセージ積算数（30 分で 500 件超でアラート）
kafka_log_log_size{namespace="kafka", topic=~".*\\.dlq"} > 500

# consumer group の lag（DLQ consumer が止まっている場合）
kafka_consumergroup_lag{namespace="kafka", topic=~".*\\.dlq"} > 0

# DLQ へのプロデュース率（上流でエラーが多発している指標）
rate(kafka_server_brokertopicmetrics_messagesinpersec{topic=~".*\\.dlq"}[5m]) > 1
```

ダッシュボード: **Grafana → k1s0 DLQ Overview**。

alert チャンネル: PagerDuty `tier1-platform-team` → Slack `#incident-kafka`。

## 2. 初動 (Immediate Action, 〜15 分)

- [ ] どの DLQ トピックが滞留しているか特定する

  ```bash
  kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
    bin/kafka-topics.sh --bootstrap-server localhost:9093 --list | grep dlq
  ```

- [ ] 各 DLQ のメッセージ数を確認する

  ```bash
  kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
    bin/kafka-log-dirs.sh --bootstrap-server localhost:9093 \
    --topic-list <dlq-topic> --describe | grep -i size
  ```

- [ ] DLQ メッセージのサンプルを取得して失敗原因を確認する

  ```bash
  kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
    bin/kafka-console-consumer.sh \
    --bootstrap-server localhost:9093 \
    --topic <dlq-topic> \
    --from-beginning --max-messages 5
  ```

- [ ] 上流 tier1 facade の error ログを確認する

  ```bash
  kubectl logs -n k1s0 deploy/tier1-facade --tail=100 | grep -i "dlq\|dead.letter\|retry"
  ```

- [ ] Dapr pubsub subscription の失敗ステータスを確認する

  ```bash
  kubectl get components -n k1s0 | grep kafka
  kubectl describe subscription -n k1s0
  ```

## 3. 復旧 (Recovery, 〜60 分)

**根本原因を修正した後、DLQ メッセージをリプレイする。**

**ステップ 1 — 根本原因を修正する**:

- コンシューマのコードバグ: 修正版をデプロイしてから DLQ をリプレイする。
- 依存サービスの一時障害: 対象サービスが復旧済みであることを確認する（PostgreSQL / SPIRE 等）。
- スキーマ不整合: Avro スキーマ変更によるデシリアライズ失敗の場合はスキーマバージョンを合わせる。

**ステップ 2 — DLQ メッセージを本来のトピックに再投入する**:

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

**ステップ 3 — リプレイ完了後、DLQ のオフセットをリセットする**:

```bash
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-consumer-groups.sh \
  --bootstrap-server localhost:9093 \
  --group <consumer-group> \
  --reset-offsets --to-latest \
  --topic <dlq-topic> --execute
```

**リプレイ進捗のモニタリング**:

```promql
kafka_consumergroup_lag{namespace="kafka", topic=~".*\\.dlq"} == 0
```

## 4. 原因調査 (Root Cause Analysis)

**Loki でのログ確認**:

```logql
{namespace="k1s0", app="tier1-facade"} |= "dlq" | json | line_format "{{.msg}}"
```

**よくある原因**:

1. **コンシューマのバグ（例外がキャッチされず retryable 扱い）**: 特定のメッセージ形式でパニックが発生し、全メッセージが DLQ に流れる。スタックトレースを `kubectl logs` で確認。
2. **依存サービス障害（DB タイムアウト / SPIRE SVID 失効）**: コンシューマ処理中に接続失敗 → 全メッセージが retry 上限に達して DLQ 転送。根本は依存先の障害対応が先決。
3. **スキーマ進化の非互換**: Proto / Avro のフィールド削除でデシリアライズ失敗。`UnknownFieldSetHandler` のログを確認。
4. **max.poll.interval.ms 超過**: 処理が遅く consumer が group から kick される。Rebalance ループで DLQ 転送が増加する。
5. **メッセージサイズ超過（1 MB デフォルト）**: `max.message.bytes` 設定と送信ペイロードのサイズを確認。

**Tempo でのトレース確認**:

```bash
# DLQ 転送時に付与された trace-id で Tempo を検索
# Grafana → Explore → Tempo → TraceID: <id>
```

## 5. 事後処理 (Post-incident)

- [ ] ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-dlq-backlog.md`）
- [ ] DLQ アラートの閾値（500 件 / 30 分）が適切か見直す
- [ ] DLQ リプレイ自動化 runbook タスクを検討（plan backlog に追加）
- [ ] コンシューマの retry 設定（`max.retries` / `backoff.ms`）を見直す
- [ ] NFR-A-REC-002 の MTTR ログを更新

## 関連

- 関連設計書: `infra/data/kafka/kafka-cluster.yaml`、`infra/dapr/components/pubsub/kafka.yaml`
- 関連 ADR: `docs/02_構想設計/adr/ADR-DATA-002`
- 関連 Runbook: `ops/runbooks/incidents/RB-MSG-001-kafka-broker-failover.md`
