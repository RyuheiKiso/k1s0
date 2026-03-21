# アラート名: PaymentDLQOverflow / PaymentHighErrorRate

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical（決済処理失敗） / warning（DLQ 蓄積） |
| **影響範囲** | 決済処理・Saga の決済ステップ |
| **通知チャネル** | Microsoft Teams #alert-critical |
| **対応 SLA** | SEV1: 15分以内（決済停止） / SEV2: 30分以内（DLQ 蓄積） |

## アラート発火条件

```promql
# DLQ メッセージ数が急増（5分間に 10 件以上増加）
increase(kafka_messages_consumed_total{
  topic=~"k1s0\\.service\\.payment\\..*\\.dlq"
}[5m]) > 10

# payment-server のエラー率が 1% 超
rate(http_requests_total{service="payment",status=~"5.."}[5m])
/ rate(http_requests_total{service="payment"}[5m]) > 0.01
```

## 初動対応（5分以内）

### 1. 状況確認

```bash
# payment-server の Pod 状態確認
kubectl get pods -n k1s0-service -l app=payment

# 直近のエラーログを確認する
kubectl logs -n k1s0-service deploy/payment --tail=100 | grep -i "error\|panic\|dlq"

# DLQ メッセージ数を確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group payment-server.default | grep dlq

# DB 接続状態を確認する
kubectl logs -n k1s0-service deploy/payment --tail=50 | grep -i "pool\|connection\|database"
```

### 2. Grafana ダッシュボード確認

- [payment サービスダッシュボード](http://grafana.k1s0.internal/d/k1s0-service/k1s0-service-dashboard?var-service=payment)
- [Kafka ダッシュボード](http://grafana.k1s0.internal/d/k1s0-kafka/kafka-dashboard)

### 3. 即時判断

- [ ] payment-server がクラッシュしている → **SEV1**（Saga の決済ステップも失敗するため即時対応）
- [ ] DLQ が急増しているが payment-server は稼働中 → **SEV2**（処理ロジックのバグ疑い）
- [ ] DLQ が緩やかに増加 → **SEV3**（原因調査、定期再処理を検討）

## 詳細調査

### よくある原因

1. **DB 接続エラー**: PostgreSQL が応答しない、コネクションプール枯渇
2. **Kafka コンシューマーのクラッシュ**: OOMKilled、パニック
3. **外部決済 API の障害**: 上流の決済プロバイダーが応答しない
4. **スキーマ非互換**: イベントメッセージの形式が変更された

### 調査コマンド

```bash
# DLQ メッセージの内容を確認して失敗原因を特定する
kubectl exec -n kafka kafka-0 -- \
  kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic k1s0.service.payment.dlq --max-messages=3 --from-beginning

# payment の DB 接続数を確認する
kubectl exec -n postgres postgres-0 -- psql -U postgres -c \
  "SELECT count(*), state, application_name FROM pg_stat_activity
   WHERE datname = 'payment_db' GROUP BY state, application_name;"

# payment-server のメモリ使用量を確認する
kubectl top pods -n k1s0-service -l app=payment
```

## 復旧手順

### パターン A: payment-server がクラッシュしている場合

```bash
# ログでパニックの原因を確認する
kubectl logs -n k1s0-service deploy/payment --previous --tail=100

# payment-server を再起動する
kubectl rollout restart deployment/payment -n k1s0-service
kubectl rollout status deployment/payment -n k1s0-service
```

### パターン B: DLQ メッセージが蓄積している場合

payment-server の復旧後、DLQ Manager から再処理する:

```bash
# DLQ メッセージの再処理（DLQ Manager API 経由）
curl -X POST http://dlq-manager.k1s0-system/api/v1/messages/retry-all \
  -H 'Content-Type: application/json' \
  -d '{"topic_pattern": "k1s0.service.payment.*.dlq"}'

# 再処理が成功しているか確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group payment-server.default
```

### パターン C: DB 接続プール枯渇の場合

[DB プール枯渇 Runbook](../common/db-pool-exhaustion.md) を参照。

```bash
# 緊急時: Pod 再起動でコネクションをリセットする
kubectl rollout restart deployment/payment -n k1s0-service
```

### パターン D: Saga と連動した障害（決済ステップが宙ぶらりん）の場合

saga-server から補償処理が自動でリトライされるか確認する:

```bash
kubectl logs -n k1s0-system deploy/saga | grep -i "payment\|compensation" | tail -20
```

自動回復しない場合は [saga Runbook](saga.md) を参照。

## エスカレーション基準

- [ ] 決済処理が完全に停止して 15 分以上経過
- [ ] 手動で補償処理が必要な Saga インスタンスが発生
- [ ] データ不整合（重複決済等）の可能性がある
- [ ] DLQ が 1,000 件を超えた

エスカレーション先: [インシデント管理設計](../../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- DLQ メッセージの `error` ヘッダーが示す失敗原因を確認
- 特定のメッセージ形式（特定の金額・通貨・ユーザー）だけ失敗していないか確認
- Saga との連携でどのステップが失敗しているか確認（Jaeger トレースを活用）

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [共通: Kafka コンシューマーラグ](../common/kafka-consumer-lag.md)
- [共通: DB プール枯渇](../common/db-pool-exhaustion.md)
- [saga Runbook](saga.md)
