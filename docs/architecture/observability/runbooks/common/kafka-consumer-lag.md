# アラート: Kafka コンシューマーラグ高騰

対象アラート: `KafkaConsumerLagHigh`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | warning |
| **影響範囲** | 対象コンシューマーグループのメッセージ処理遅延 |
| **通知チャネル** | Microsoft Teams #alert-warning |
| **対応 SLA** | SEV2（30分以内） / ラグが急増中は SEV1 へ昇格 |

## アラート発火条件

- コンシューマーグループのラグ合計 > 1000 で 5分継続

## 初動対応（5分以内）

### 1. ラグの状況を確認

```bash
# コンシューマーグループのラグ確認
kubectl exec -n kafka deploy/kafka -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group {consumer-group-name}

# 全グループの概要確認
kubectl exec -n kafka deploy/kafka -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 --list
```

Grafana → Kafka ダッシュボード → Consumer Lag パネルを確認:
- [Kafka ダッシュボード](http://grafana.k1s0.internal/d/k1s0-kafka/kafka-dashboard)

### 2. ラグの増加傾向を確認

- [ ] ラグが増加し続けている → SEV1 へ昇格（コンシューマーが停止している可能性）
- [ ] ラグが一時的に増えたが安定している → SEV2（詳細調査）
- [ ] ラグが減少傾向にある → SEV3（監視継続）

## 詳細調査

### よくある原因

1. **コンシューマーの処理速度不足**: 処理が重くなり消費ペースが低下
2. **コンシューマーの停止**: Pod クラッシュやスケールダウンによる消費者減少
3. **プロデューサー側の急増**: イベント発生量が急増（例: バッチ処理）
4. **コンシューマーリバランス**: Pod 再起動によるリバランス中の停止

### 調査コマンド

```bash
# コンシューマーアプリの Pod 状態確認
kubectl get pods -n {namespace} | grep {consumer-service-name}

# コンシューマーログの確認
kubectl logs -n {namespace} deploy/{consumer-service-name} --tail=100 | grep -i "kafka\|consumer\|lag\|error"

# トピック別のラグ内訳
kubectl exec -n kafka deploy/kafka -- \
  kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group {consumer-group-name} | sort -k5 -rn
```

### Prometheus クエリ例

```promql
# コンシューマーグループ別ラグの推移
kafka_consumergroup_lag_sum{namespace=~"k1s0-.*"}

# ラグの変化率（増加中かどうか）
rate(kafka_consumergroup_lag_sum{consumergroup="{group-name}"}[10m])
```

## 復旧手順

### パターン A: コンシューマーが停止している場合

```bash
# コンシューマーサービスの再起動
kubectl rollout restart deployment/{consumer-service-name} -n {namespace}

# Pod が正常に起動して消費を再開しているか確認
kubectl logs -n {namespace} deploy/{consumer-service-name} -f | grep "consuming"
```

### パターン B: 処理速度不足

```bash
# コンシューマーをスケールアウト（パーティション数以下に制限）
kubectl scale deployment/{consumer-service-name} -n {namespace} --replicas={n}
```

### パターン C: リバランス中

```bash
# リバランスが完了するまで待機（通常 1〜2分）
# 完了後もラグが増え続ける場合はパターン A/B を確認
```

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] ラグが 10000 を超えて増加し続けている
- [ ] コンシューマーサービスが CrashLoopBackOff 状態
- [ ] 再起動・スケールアウト後も改善しない
- [ ] Kafka クラスター自体に問題がある可能性

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- 処理ロジックに時間のかかる同期処理がないか確認
- デプロイによってコンシューマー数が減っていないか確認
- プロデューサー側でイベント量が増加していないか確認（バッチ処理のスケジュール等）

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
