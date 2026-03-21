# アラート名: OrderKafkaConsumerLag / OrderHighErrorRate

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical（注文受付停止） / warning（処理遅延） |
| **影響範囲** | 注文作成・更新処理、Saga トリガー |
| **通知チャネル** | Microsoft Teams #alert-critical / #alert-warning |
| **対応 SLA** | SEV1: 15分以内 / SEV2: 30分以内 |

## アラート発火条件

```promql
# 注文イベントのコンシューマーラグが 100 を超えた
kafka_consumergroup_lag_sum{
  consumer_group="order-server.default"
} > 100

# order-server のエラー率が 1% 超
rate(http_requests_total{service="order",status=~"5.."}[5m])
/ rate(http_requests_total{service="order"}[5m]) > 0.01
```

## 初動対応（5分以内）

### 1. 状況確認

```bash
# order-server の Pod 状態確認
kubectl get pods -n k1s0-service -l app=order

# コンシューマーラグを確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group order-server.default

# 直近のエラーログを確認する
kubectl logs -n k1s0-service deploy/order --tail=100 | grep -i "error\|kafka\|consumer"
```

### 2. Grafana ダッシュボード確認

- [order サービスダッシュボード](http://grafana.k1s0.internal/d/k1s0-service/k1s0-service-dashboard?var-service=order)
- [Kafka ダッシュボード](http://grafana.k1s0.internal/d/k1s0-kafka/kafka-dashboard)

### 3. 即時判断

- [ ] ラグが増加し続け、コンシューマーが停止している → **SEV1**
- [ ] ラグが増加しているが消費は継続している（処理速度が追いつかない） → **SEV2**
- [ ] ラグが自然に減少している → **SEV3**（リバランス中、経過観察）

## 詳細調査

### よくある原因

1. **コンシューマーのクラッシュ**: OOMKilled、パニック、プロセス停止
2. **処理速度の不足**: DB 負荷増加、ロングクエリ
3. **Kafka ブローカーの問題**: リーダー切り替え、ネットワーク分断
4. **イベントスキーマの非互換**: メッセージ形式の変更で処理できないイベントが発生

### 調査コマンド

```bash
# コンシューマーが OOMKilled されていないか確認する
kubectl describe pods -n k1s0-service -l app=order | grep -A 5 "OOMKilled\|Reason"

# order-server のメモリ・CPU 使用量を確認する
kubectl top pods -n k1s0-service -l app=order

# Kafka ブローカーの状態を確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-broker-api-versions.sh --bootstrap-server localhost:9092 2>&1 | head -5

# 処理できないメッセージ（DLQ 行き）を確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic k1s0.service.order.created.v1.dlq --max-messages=3 --from-beginning
```

## 復旧手順

### パターン A: コンシューマーが停止している場合

```bash
# ログで停止原因を確認する
kubectl logs -n k1s0-service deploy/order --previous --tail=100

# order-server を再起動する
kubectl rollout restart deployment/order -n k1s0-service
kubectl rollout status deployment/order -n k1s0-service

# 再起動後にラグが減少しているか確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group order-server.default
```

### パターン B: 処理速度が不足している場合

```bash
# レプリカ数を増やしてスケールアウトする（Kafka パーティション数を超えないよう注意）
kubectl scale deployment/order -n k1s0-service --replicas=3
```

### パターン C: DLQ にメッセージが溜まっている場合

```bash
# DLQ メッセージを再処理する（原因修正・再デプロイ後に実行）
curl -X POST http://dlq-manager.k1s0-system/api/v1/messages/retry-all \
  -H 'Content-Type: application/json' \
  -d '{"topic_pattern": "k1s0.service.order.*.dlq"}'
```

## エスカレーション基準

- [ ] 注文受付が完全に停止して 15 分以上経過
- [ ] ラグが 10,000 件を超えた
- [ ] Saga トリガーが止まり在庫・決済処理も停止した
- [ ] データ不整合（注文の二重作成等）の可能性がある

エスカレーション先: [インシデント管理設計](../../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- ラグが増え始めた時刻とデプロイ履歴が一致するか確認
- DLQ メッセージの `error` ヘッダーで失敗原因を特定
- 特定のトピック（`order.created` か `order.cancelled` 等）のみ問題か確認

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [共通: Kafka コンシューマーラグ](../common/kafka-consumer-lag.md)
- [共通: DB プール枯渇](../common/db-pool-exhaustion.md)
- [payment Runbook](payment.md)
- [saga Runbook](saga.md)
