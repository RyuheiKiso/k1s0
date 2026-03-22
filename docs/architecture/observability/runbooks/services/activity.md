# アラート名: ActivityDLQOverflow / ActivityHighErrorRate

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical（アクティビティ処理失敗） / warning（DLQ 蓄積） |
| **影響範囲** | アクティビティ処理・Saga のアクティビティステップ |
| **通知チャネル** | Microsoft Teams #alert-critical |
| **対応 SLA** | SEV1: 15分以内（アクティビティ停止） / SEV2: 30分以内（DLQ 蓄積） |

## アラート発火条件

```promql
# DLQ メッセージ数が急増（5分間に 10 件以上増加）
increase(kafka_messages_consumed_total{
  topic=~"k1s0\\.service\\.activity\\..*\\.dlq"
}[5m]) > 10

# activity-server のエラー率が 1% 超
rate(http_requests_total{service="activity",status=~"5.."}[5m])
/ rate(http_requests_total{service="activity"}[5m]) > 0.01
```

## 初動対応（5分以内）

### 1. 状況確認

```bash
# activity-server の Pod 状態確認
kubectl get pods -n k1s0-service -l app=activity

# 直近のエラーログを確認する
kubectl logs -n k1s0-service deploy/activity --tail=100 | grep -i "error\|panic\|dlq"

# DLQ メッセージ数を確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group activity-server.default | grep dlq

# DB 接続状態を確認する
kubectl logs -n k1s0-service deploy/activity --tail=50 | grep -i "pool\|connection\|database"
```

### 2. Grafana ダッシュボード確認

- [activity サービスダッシュボード](http://grafana.k1s0.internal/d/k1s0-service/k1s0-service-dashboard?var-service=activity)
- [Kafka ダッシュボード](http://grafana.k1s0.internal/d/k1s0-kafka/kafka-dashboard)

### 3. 即時判断

- [ ] activity-server がクラッシュしている → **SEV1**（Saga のアクティビティステップも失敗するため即時対応）
- [ ] DLQ が急増しているが activity-server は稼働中 → **SEV2**（処理ロジックのバグ疑い）
- [ ] DLQ が緩やかに増加 → **SEV3**（原因調査、定期再処理を検討）

## 詳細調査

### よくある原因

1. **DB 接続エラー**: PostgreSQL が応答しない、コネクションプール枯渇
2. **Kafka コンシューマーのクラッシュ**: OOMKilled、パニック
3. **外部連携 API の障害**: 上流の連携先サービスが応答しない
4. **スキーマ非互換**: イベントメッセージの形式が変更された

### 調査コマンド

```bash
# DLQ メッセージの内容を確認して失敗原因を特定する
kubectl exec -n kafka kafka-0 -- \
  kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic k1s0.service.activity.dlq --max-messages=3 --from-beginning

# activity の DB 接続数を確認する
kubectl exec -n postgres postgres-0 -- psql -U postgres -c \
  "SELECT count(*), state, application_name FROM pg_stat_activity
   WHERE datname = 'activity_db' GROUP BY state, application_name;"

# activity-server のメモリ使用量を確認する
kubectl top pods -n k1s0-service -l app=activity
```

## 復旧手順

### パターン A: activity-server がクラッシュしている場合

```bash
# ログでパニックの原因を確認する
kubectl logs -n k1s0-service deploy/activity --previous --tail=100

# activity-server を再起動する
kubectl rollout restart deployment/activity -n k1s0-service
kubectl rollout status deployment/activity -n k1s0-service
```

### パターン B: DLQ メッセージが蓄積している場合

activity-server の復旧後、DLQ Manager から再処理する:

```bash
# DLQ メッセージの再処理（DLQ Manager API 経由）
curl -X POST http://dlq-manager.k1s0-system/api/v1/messages/retry-all \
  -H 'Content-Type: application/json' \
  -d '{"topic_pattern": "k1s0.service.activity.*.dlq"}'

# 再処理が成功しているか確認する
kubectl exec -n kafka kafka-0 -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group activity-server.default
```

### パターン C: DB 接続プール枯渇の場合

[DB プール枯渇 Runbook](../common/db-pool-exhaustion.md) を参照。

```bash
# 緊急時: Pod 再起動でコネクションをリセットする
kubectl rollout restart deployment/activity -n k1s0-service
```

### パターン D: Saga と連動した障害（アクティビティステップが宙ぶらりん）の場合

saga-server から補償処理が自動でリトライされるか確認する:

```bash
kubectl logs -n k1s0-system deploy/saga | grep -i "activity\|compensation" | tail -20
```

自動回復しない場合は [saga Runbook](saga.md) を参照。

## エスカレーション基準

- [ ] アクティビティ処理が完全に停止して 15 分以上経過
- [ ] 手動で補償処理が必要な Saga インスタンスが発生
- [ ] データ不整合（重複アクティビティ等）の可能性がある
- [ ] DLQ が 1,000 件を超えた

エスカレーション先: [インシデント管理設計](../../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- DLQ メッセージの `error` ヘッダーが示す失敗原因を確認
- 特定のメッセージ形式（特定のユーザー・テナント）だけ失敗していないか確認
- Saga との連携でどのステップが失敗しているか確認（Jaeger トレースを活用）

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [共通: Kafka コンシューマーラグ](../common/kafka-consumer-lag.md)
- [共通: DB プール枯渇](../common/db-pool-exhaustion.md)
- [saga Runbook](saga.md)
