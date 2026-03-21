# アラート名: config サービス障害

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical |
| **影響範囲** | 全サービス（設定取得ができなくなると起動・動作に影響） |
| **通知チャネル** | Microsoft Teams #alert-critical |
| **対応 SLA** | SEV1: 15分以内 / SEV2: 30分以内 |

## アラート発火条件

```promql
# サービスダウン
up{job="config"} == 0

# エラーレート高騰
sum(rate(http_requests_total{job="config",status=~"5.."}[5m]))
  / sum(rate(http_requests_total{job="config"}[5m])) > 0.05

# 設定伝搬の遅延（config サービスのカスタムメトリクス）
config_propagation_lag_seconds > 30
```

## 初動対応（5分以内）

### 1. 状況確認

```bash
# Pod の状態確認
kubectl get pods -n k1s0-system -l app=config

# 直近のエラーログ確認
# Grafana → Explore → Loki: {app="config", level="error"} | 直近10分

# DB 接続確認（config_db への疎通）
kubectl exec -n k1s0-system deploy/config -- \
  wget -qO- http://127.0.0.1:8080/healthz

# 設定値の取得が動作しているか確認する
kubectl exec -n k1s0-system deploy/config -- \
  wget -qO- "http://127.0.0.1:8080/api/v1/configs/test-key"
```

### 2. Grafana ダッシュボード確認

- [config サービスダッシュボード](http://grafana.k1s0.internal/d/k1s0-config/config-dashboard)
- [SLO ダッシュボード](http://grafana.k1s0.internal/d/k1s0-slo/slo-dashboard)

### 3. 即時判断

- [ ] config Pod が全て落ちている → SEV1
- [ ] DB 接続が失敗している → SEV1（DB 障害）
- [ ] 設定伝搬が遅延している → SEV2（キャッシュ不整合の可能性）
- [ ] 一部 Pod のみ障害 → SEV2

## 詳細調査

### よくある原因

1. **DB 接続エラー**: config_db への接続失敗（DB 障害、接続プール枯渇）
2. **キャッシュ不整合**: Redis キャッシュと DB の値が乖離し、設定変更が伝搬されない
3. **設定値の不正**: 不正な設定値が投入され、バリデーションエラーで処理が停止
4. **OOM Kill**: 設定データ量が多い環境でのメモリ不足

### 調査コマンド

```bash
# DB 接続プールの状態確認
kubectl exec -n k1s0-system deploy/config -- \
  wget -qO- http://127.0.0.1:8080/metrics | grep db_pool

# キャッシュの状態確認（Redis）
kubectl exec -n k1s0-system deploy/redis -- \
  redis-cli -h redis.k1s0-system.svc.cluster.local info keyspace

# 設定伝搬ラグのメトリクス確認
# Prometheus: config_propagation_lag_seconds{app="config"}

# DB の接続数確認
kubectl exec -n k1s0-system deploy/config -- \
  psql -h config-db.k1s0-system.svc.cluster.local \
       -U config_user -d config_db \
       -c "SELECT count(*) FROM pg_stat_activity WHERE datname='config_db';"
```

## 復旧手順

### パターン A: Pod 障害の場合

```bash
kubectl rollout restart deployment/config -n k1s0-system
kubectl rollout status deployment/config -n k1s0-system
```

### パターン B: キャッシュ不整合の場合

```bash
# Redis キャッシュをフラッシュする（影響: 次のリクエストまでDB直接参照になる）
kubectl exec -n k1s0-system deploy/redis -- \
  redis-cli -h redis.k1s0-system.svc.cluster.local FLUSHDB

# キャッシュクリア後、config を再起動して再ウォームアップする
kubectl rollout restart deployment/config -n k1s0-system
```

### パターン C: 不正設定値の場合

```bash
# 最後に投入された設定変更を確認する
# Grafana → Loki: {app="config"} | "validation_error" | 直近30分

# 問題のある設定値を特定してロールバックする
# config サービスのAPI経由で前の値に戻す
kubectl exec -n k1s0-system deploy/config -- \
  curl -X PUT "http://127.0.0.1:8080/api/v1/configs/<key>" \
    -H "Content-Type: application/json" \
    -d '{"value": "<previous_value>"}'
```

## エスカレーション基準

- [ ] config Pod が全台 CrashLoopBackOff から回復しない
- [ ] DB 障害が原因で DB チームへのエスカレーションが必要
- [ ] 15分以内に復旧の見通しが立たない

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- 設定伝搬ラグの閾値（30秒）が適切か定期的に見直す
- DB 接続プールの `max_open_conns` と実際の接続数の関係を確認する
- 設定値バリデーションのルールが本番データに対して適切か確認する

## 関連ドキュメント

- [db-pool-exhaustion Runbook](../common/db-pool-exhaustion.md)
- [デプロイ手順書](../../../infrastructure/kubernetes/デプロイ手順書.md)
- [インシデント管理設計](../インシデント管理設計.md)
