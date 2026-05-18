# Quota Enforcement Failure Runbook

## Classification
- incident_class: quota_enforcement_failure
- severity: P2 (通常) → P1 (隣接テナントへのリソース侵食が確認された場合)
- spec_reference: docs/04_詳細設計/01_適合仕様/09_テナント容量適合仕様.md

## Phase 1: Detection (検知)
1. Prometheus アラート `TenantQuotaOverflow` を確認する
2. 影響テナントと超過量を確認する
   - Envoy local rate limit メトリクス: `envoy_local_rate_limit_rate_limited_total`
   - Kafka quota メトリクス: `kafka_quota_throttle_time_avg`
3. Grafana の per-tenant QPS / throughput ダッシュボードを確認する
4. 隣接テナントへの影響 (noisy neighbor) が発生していないか確認する

## Phase 2: Containment (封じ込め)
1. quota 超過テナントへの接続を一時的に絞る
   - Envoy BackendTrafficPolicy の rate limit を緊急引き下げる
2. Kafka quota を超過テナントのみ一時的に制限する
   - `kafka-configs.sh --bootstrap-server kafka:9092 --alter --add-config 'producerByteRate=1048576' --entity-type users --entity-name <tenant-user>`
3. 影響を受けた隣接テナントへの QoS を回復する
4. per_tenant_quota.yaml の現在の設定値を記録する

## Phase 3: Investigation (調査)
1. quota 超過テナントのリクエストパターンを分析する
   - burst なのか sustained increase なのかを判別する
2. テナントの利用規約違反の可能性を確認する
3. Envoy local_rate_limit_filter.yaml の設定が正しく適用されているか確認する
   - `kubectl get backendtrafficpolicy k1s0-per-tenant-rate-limit -o yaml`
4. KEDA の autoscale が正しく動作しているか確認する

## Phase 4: Resolution (解決)
1. 正当なトラフィック増加の場合: テナントと協議して quota を増加する
   - per_tenant_quota.yaml と local_rate_limit_filter.yaml を更新する
2. 悪意あるトラフィックの場合: テナントアカウントを一時停止する
3. quota enforcement の設定漏れを修正する
4. Kafka KafkaUser リソースを更新して新 quota を適用する

## Phase 5: Recovery Verification (回復確認)
1. 隣接テナントへの影響が解消されたことを確認する
2. 全テナントの QPS が quota 以内に収まっていることを確認する
3. Litmus stress_test シナリオ `s06_neighbor_isolation` を実行して回帰確認する
4. Envoy と Kafka 両方の quota メトリクスが正常範囲に戻ったことを確認する

## Phase 6: Postmortem (事後分析)
1. quota enforcement の gap があった場合はその原因を分析する
2. per-tenant quota の設定値が適切かを見直す
3. quota 超過の早期警告 (75% 使用時) アラートを追加する
4. stress_test シナリオ s01-s08 を全件実行して回帰がないことを確認する
