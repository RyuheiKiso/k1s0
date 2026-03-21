# アラート名: saga サービス障害

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical / warning |
| **影響範囲** | 分散トランザクション（注文・決済等の業務フロー全体） |
| **通知チャネル** | Microsoft Teams #alert-critical / #alert-warning |
| **対応 SLA** | SEV1: 15分以内 / SEV2: 30分以内 |

## アラート発火条件

```promql
# サービスダウン
up{job="saga"} == 0

# DLQ 滞留（補償失敗メッセージが蓄積している）
kafka_consumer_group_lag{group="saga-dlq-consumer"} > 100

# 補償トランザクション失敗レート
rate(saga_compensation_failures_total[5m]) > 1

# Saga タイムアウト（長時間実行中の Saga が増加）
saga_active_count{status="running"} > 50
```

## 初動対応（5分以内）

### 1. 状況確認

```bash
# Pod の状態確認
kubectl get pods -n k1s0-system -l app=saga

# Kafka コンシューマーラグの確認
kubectl exec -n messaging deploy/kafka-client -- \
  kafka-consumer-groups.sh \
    --bootstrap-server kafka.messaging.svc.cluster.local:9092 \
    --group saga-consumer \
    --describe

# DLQ のメッセージ数確認
kubectl exec -n messaging deploy/kafka-client -- \
  kafka-consumer-groups.sh \
    --bootstrap-server kafka.messaging.svc.cluster.local:9092 \
    --group saga-dlq-consumer \
    --describe

# 実行中の Saga 一覧（Saga ストア DB を直接確認）
kubectl exec -n k1s0-system deploy/saga -- \
  psql -h saga-db.k1s0-system.svc.cluster.local \
       -U saga_user -d saga_db \
       -c "SELECT id, status, created_at, updated_at FROM sagas WHERE status='running' ORDER BY created_at LIMIT 20;"
```

### 2. Grafana ダッシュボード確認

- [saga サービスダッシュボード](http://grafana.k1s0.internal/d/k1s0-saga/saga-dashboard)
- [Kafka コンシューマーラグ](http://grafana.k1s0.internal/d/k1s0-kafka/kafka-dashboard)
- [SLO ダッシュボード](http://grafana.k1s0.internal/d/k1s0-slo/slo-dashboard)

### 3. 即時判断

- [ ] saga Pod が全て落ちている → SEV1（即時エスカレーション）
- [ ] Kafka が応答しない → SEV1（Kafka 障害）
- [ ] DLQ に大量メッセージが滞留 → SEV2（補償トランザクション失敗）
- [ ] Saga タイムアウトが増加 → SEV2（下流サービス障害の可能性）

## 詳細調査

### よくある原因

1. **補償トランザクション失敗**: 下流サービス（order/payment/inventory）が補償アクションを拒否
2. **DLQ 滞留**: 補償失敗メッセージが DLQ に蓄積し、dlq-manager が処理できていない
3. **Kafka 接続断**: Kafka ブローカーへの接続が切れ、メッセージ受信が停止
4. **Saga タイムアウト**: 外部サービスの応答遅延により Saga が長時間 running 状態に
5. **DB 接続枯渇**: Saga ストアへの接続プールが枯渇

### 調査コマンド

```bash
# DLQ のメッセージ内容を確認する（最新10件）
kubectl exec -n messaging deploy/kafka-client -- \
  kafka-console-consumer.sh \
    --bootstrap-server kafka.messaging.svc.cluster.local:9092 \
    --topic saga-dlq \
    --from-beginning \
    --max-messages 10

# 失敗した補償トランザクションの詳細
kubectl exec -n k1s0-system deploy/saga -- \
  psql -h saga-db.k1s0-system.svc.cluster.local \
       -U saga_user -d saga_db \
       -c "SELECT id, saga_id, step_name, error_message, retry_count FROM saga_compensations WHERE status='failed' ORDER BY created_at DESC LIMIT 20;"

# タイムアウトした Saga の確認
kubectl exec -n k1s0-system deploy/saga -- \
  psql -h saga-db.k1s0-system.svc.cluster.local \
       -U saga_user -d saga_db \
       -c "SELECT id, created_at, updated_at, EXTRACT(EPOCH FROM (NOW()-updated_at)) AS idle_seconds FROM sagas WHERE status='running' ORDER BY idle_seconds DESC LIMIT 10;"
```

## 復旧手順

### パターン A: saga Pod 障害の場合

```bash
kubectl rollout restart deployment/saga -n k1s0-system
kubectl rollout status deployment/saga -n k1s0-system
```

### パターン B: DLQ 滞留（補償失敗）の場合

```bash
# 1. dlq-manager の状態を確認する
kubectl get pods -n k1s0-system -l app=dlq-manager
kubectl logs -n k1s0-system deploy/dlq-manager --tail=100

# 2. DLQ メッセージを手動で再キューイングする（下流サービスが復旧後）
# dlq-manager の管理 API を使用する
kubectl exec -n k1s0-system deploy/dlq-manager -- \
  curl -X POST "http://127.0.0.1:8080/api/v1/dlq/requeue-all"

# 3. 再キューイングが難しい場合は下流サービスのチームにエスカレーションする
```

### パターン C: Saga タイムアウトの場合

```bash
# 1. 原因となっている下流サービスを特定する
# Grafana → Jaeger → saga サービスのトレース → 遅延しているサービスを確認

# 2. 下流サービスの Runbook を参照して復旧する

# 3. タイムアウトした Saga を手動でキャンセルする（業務チームの確認後）
kubectl exec -n k1s0-system deploy/saga -- \
  psql -h saga-db.k1s0-system.svc.cluster.local \
       -U saga_user -d saga_db \
       -c "UPDATE sagas SET status='cancelled', updated_at=NOW() WHERE id='<saga_id>';"
```

### パターン D: Kafka 接続断の場合

[kafka-consumer-lag Runbook](../common/kafka-consumer-lag.md) を参照して Kafka を復旧する。

## エスカレーション基準

- [ ] 補償トランザクションが5件以上失敗し、DLQ に滞留している
- [ ] Kafka 障害が原因で Kafka チームへのエスカレーションが必要
- [ ] タイムアウトした Saga が業務データに影響している（業務チームへの連絡が必要）
- [ ] 15分以内に復旧の見通しが立たない

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- 補償失敗の原因となった下流サービスの障害パターンを分析する
- Saga タイムアウト閾値が下流サービスのP99レイテンシに対して適切か確認する
- DLQ の最大保持件数と処理レートの設定を見直す

## 関連ドキュメント

- [kafka-consumer-lag Runbook](../common/kafka-consumer-lag.md)
- [デプロイ手順書](../../../infrastructure/kubernetes/デプロイ手順書.md)
- [インシデント管理設計](../インシデント管理設計.md)
