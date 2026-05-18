# Kafka Consumer Lag Runbook

## Classification
- incident_class: kafka_consumer_lag
- severity: P2 (初期) → P1 (lag > 100k messages)
- spec_reference: docs/04_詳細設計/01_適合仕様/15_読み取りモデル方針適合仕様.md

## Phase 1: Detection (検知)
1. Prometheus アラート `KafkaConsumerLagHigh` を確認する
2. Kafka consumer group の lag を確認する: `kafka-consumer-groups.sh --bootstrap-server kafka:9092 --describe --all-groups`
3. 影響 topic と consumer group を特定する
4. lag 増加速度を Grafana で確認する (急増 vs 緩増)

## Phase 2: Containment (封じ込め)
1. lag の原因が producer 増加か consumer 低下かを判別する
2. producer 増加が原因の場合: Envoy rate limit を一時的に強化する
3. consumer 低下が原因の場合: consumer pod の状態を確認する
   - `kubectl get pods -n tier2 -l app=tier2-consumer`
4. Dead Letter Queue (DLQ) の滞留を確認する

## Phase 3: Investigation (調査)
1. consumer pod のログを確認する: `kubectl logs -n tier2 -l app=tier2-consumer --tail=200`
2. consumer の処理時間 (p99) を Jaeger トレースで確認する
3. PostgreSQL のスロークエリログを確認する (consumer が DB に依存している場合)
4. KEDA ScaledObject の設定を確認する: `kubectl get scaledobject -n tier2`

## Phase 4: Resolution (解決)
1. consumer の水平スケールアウトを実施する
   - `kubectl scale deployment tier2-consumer --replicas=<new_count> -n tier2`
2. KEDA の min/max replicas を調整する
3. consumer の処理ボトルネックを特定してコードレベルで修正する
4. Kafka per-tenant quota (per_tenant_quota.yaml) を見直す

## Phase 5: Recovery Verification (回復確認)
1. consumer lag が正常範囲 (< 1000 messages) に戻ったことを確認する
2. Debezium outbox connector の lag も確認する
3. domain_event の投影遅延が SLO 以内に戻ったことを確認する
4. per-tenant quota の不公平分配がないことを確認する

## Phase 6: Postmortem (事後分析)
1. lag 発生の根本原因を明記する
2. KEDA autoscale が機能しなかった場合はその原因を分析する
3. per_tenant_quota.yaml の producerByteRate / consumerByteRate を見直す
4. 再発防止のための monitoring 閾値を調整する
