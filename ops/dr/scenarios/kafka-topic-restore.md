# kafka-topic-restore — Kafka トピック消失からの復旧

本シナリオは Kafka トピックの誤削除や partition データ喪失時の復旧手順を定める。
RTO: 30 分。RPO: 採用後の運用拡大時で MirrorMaker 別クラスタ運用時のみ復旧可能（リリース時点 では topic 再作成 + 上流再送のみ）。

## 1. 前提条件

- Kafka cluster `k1s0-kafka` が起動中（broker 障害時は [`../../runbooks/incidents/RB-MSG-001`](../../runbooks/incidents/RB-MSG-001-kafka-broker-failover.md) 先行）。
- 採用後の運用拡大時で MirrorMaker 2 で別クラスタにレプリカが存在すること（リリース時点 は単一クラスタ運用）。

## 2. シナリオ

例: tier1 facade のトピック `tier1-events` が誤って `kafka-topics --delete` された。
影響: PubSub publish 失敗、tier1 → tier2 イベント連携停止。

## 3. 復旧手順

### リリース時点（単一クラスタ運用）

```bash
# 1. トピック再作成
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-topics.sh --bootstrap-server localhost:9093 \
  --create --topic tier1-events \
  --partitions 12 --replication-factor 3 \
  --config retention.ms=604800000

# 2. 上流アプリにイベント再送を依頼（業務影響を顧客に通知）
# 3. DLQ に滞留中のメッセージを再投入（[`RB-MSG-002`](../../runbooks/incidents/RB-MSG-002-dlq-backlog.md)）
```

### 採用後の運用拡大時（MirrorMaker 別クラスタ運用時）

```bash
# 1. MirrorMaker 2 で source（DR cluster）→ target（本番）にミラー
kubectl apply -f - <<EOF
apiVersion: kafka.strimzi.io/v1beta2
kind: KafkaMirrorMaker2
metadata:
  name: k1s0-kafka-mm2-dr-recovery
  namespace: kafka
spec:
  version: 3.7.0
  replicas: 1
  connectCluster: "k1s0-kafka"
  clusters:
    - alias: "dr"
      bootstrapServers: k1s0-kafka-dr-bootstrap.kafka-dr.svc:9093
    - alias: "k1s0-kafka"
      bootstrapServers: k1s0-kafka-bootstrap.kafka.svc:9093
  mirrors:
    - sourceCluster: "dr"
      targetCluster: "k1s0-kafka"
      sourceConnector:
        config:
          replication.factor: "3"
          offset-syncs.topic.replication.factor: "3"
      topicsPattern: "tier1-events"
EOF
```

## 4. 検証

- `kubectl exec ... kafka-topics.sh --describe --topic tier1-events` で topic 存在確認。
- partition 数 / replication factor が想定通り。
- tier1 facade の publish が成功（Loki クエリで確認）。
- DLQ が解消（`kafka_consumergroup_lag{topic=~".*\\.dlq"} == 0`）。

## 5. 関連

- 関連 Runbook: [`../../runbooks/incidents/RB-MSG-001-kafka-broker-failover.md`](../../runbooks/incidents/RB-MSG-001-kafka-broker-failover.md), [`../../runbooks/incidents/RB-MSG-002-dlq-backlog.md`](../../runbooks/incidents/RB-MSG-002-dlq-backlog.md)
