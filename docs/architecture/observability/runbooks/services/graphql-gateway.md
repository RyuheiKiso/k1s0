# アラート名: GraphQLGatewayBackendTimeout / GraphQLGatewayHighErrorRate

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical（全バックエンド影響時） / warning（一部バックエンド） |
| **影響範囲** | フロントエンド経由の全 GraphQL 操作 |
| **通知チャネル** | Microsoft Teams #alert-critical / #alert-warning |
| **対応 SLA** | SEV1: 15分以内 / SEV2: 30分以内 |

## アラート発火条件

```promql
# バックエンドタイムアウト率が 5% 超
rate(grpc_client_handled_total{
  service="graphql-gateway",
  grpc_code="DeadlineExceeded"
}[5m])
/ rate(grpc_client_handled_total{service="graphql-gateway"}[5m]) > 0.05

# P99 レイテンシが 5 秒超
histogram_quantile(0.99,
  rate(http_request_duration_seconds_bucket{service="graphql-gateway"}[5m])
) > 5
```

- 閾値: タイムアウト率 > 5%、またはP99 > 5秒
- 継続時間: `for: 5m`

## 初動対応（5分以内）

### 1. 状況確認

```bash
# graphql-gateway の Pod 状態確認
kubectl get pods -n k1s0-system -l app=graphql-gateway

# タイムアウトログを確認してどのバックエンドが遅いか特定する
kubectl logs -n k1s0-system deploy/graphql-gateway --tail=100 | grep -i "timeout\|deadline\|upstream"

# 各バックエンドへの疎通確認
for svc in auth session tenant featureflag config navigation service-catalog vault scheduler notification workflow; do
  echo -n "=== $svc: "
  kubectl exec -n k1s0-system deploy/graphql-gateway -- \
    grpc_health_probe -addr="$svc-rust:50051" 2>&1 || echo "UNREACHABLE"
done
```

### 2. Grafana / Jaeger で原因バックエンドを特定

- [graphql-gateway ダッシュボード](http://grafana.k1s0.internal/d/k1s0-service/k1s0-service-dashboard?var-service=graphql-gateway)
- [Jaeger: 1 秒以上のトレースを検索](http://jaeger.k1s0.internal) → Service: graphql-gateway, Min Duration: 1s

### 3. 即時判断

- [ ] 全バックエンドが応答不能 → **SEV1**（graphql-gateway 自体の障害、または Kubernetes ネットワーク障害）
- [ ] 特定のバックエンドが遅い → **SEV2**（そのバックエンドの Runbook を参照）
- [ ] P99 が上昇しているが機能停止ではない → **SEV3**（監視継続・バックエンドの詳細調査）

## 詳細調査

### よくある原因

1. **特定のバックエンドサービスの遅延**: DB 負荷増加、メモリ不足、ロングクエリ等
2. **バックエンドサービスのクラッシュ**: CrashLoopBackOff、OOMKilled
3. **Kubernetes ネットワーク分断**: Pod 間通信の遮断
4. **graphql-gateway 自体のメモリ不足**: メモリリーク、N+1 クエリによる過負荷

### 調査コマンド

```bash
# 遅延バックエンドを Prometheus で特定する（gRPC レイテンシ）
# Grafana → Explore → Prometheus
histogram_quantile(0.99,
  rate(grpc_client_handling_seconds_bucket{service="graphql-gateway"}[5m])
) by (grpc_service)

# graphql-gateway のメモリ使用量を確認
kubectl top pods -n k1s0-system -l app=graphql-gateway

# 各バックエンドの Pod ステータスを一括確認
for svc in auth-rust session-rust tenant-rust featureflag-rust config-rust \
           navigation-rust service-catalog-rust vault-rust scheduler-rust \
           notification-rust workflow-rust; do
  echo "=== $svc ==="
  kubectl get pods -n k1s0-system -l app=$svc
done
```

## 復旧手順

### パターン A: 特定のバックエンドが遅い・ダウンしている場合

```bash
# 該当バックエンドを再起動（例: session-server が原因の場合）
kubectl rollout restart deployment/session-rust -n k1s0-system

# 起動完了を確認
kubectl rollout status deployment/session-rust -n k1s0-system
```

詳細は該当サービスの Runbook を参照:
- auth → [auth Runbook](auth.md)
- session → [session Runbook](session.md)（未作成の場合は [サービスダウン共通 Runbook](../common/service-down.md)）

### パターン B: graphql-gateway 自体が過負荷の場合

```bash
# スケールアウト
kubectl scale deployment/graphql-gateway -n k1s0-system --replicas=3

# graphql-gateway を再起動（メモリリーク疑いの場合）
kubectl rollout restart deployment/graphql-gateway -n k1s0-system
```

### パターン C: タイムアウト値の一時的調整

遅いバックエンドのタイムアウトを個別に調整することで他クエリへの波及を抑制できる。

> **既知の制限**: バックエンド別タイムアウトは `config.docker.yaml` / `config.prod.yaml` の
> `backends.{name}.timeout_ms` で設定可能。変更後は Pod 再起動が必要。

```bash
# 設定変更後に再起動
kubectl rollout restart deployment/graphql-gateway -n k1s0-system
```

## エスカレーション基準

以下の条件に該当する場合は上位にエスカレーションする:

- [ ] 15分以内に復旧の見通しが立たない
- [ ] 全バックエンドが同時に応答不能（Kubernetes インフラ障害の可能性）
- [ ] SLO バーンレートが critical レベルを超えた

エスカレーション先: [インシデント管理設計](../../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- どのバックエンドで問題が発生したか（Jaeger トレースが最も有効）
- バックエンドの DB クエリが遅かったか（[DB プール枯渇 Runbook](../common/db-pool-exhaustion.md) も確認）
- タイムアウト値が適切か（遅いサービスには長め、非クリティカルには短めを設定）

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [監視アラート設計](../../監視アラート設計.md)
- [SLO 設計](../../SLO設計.md)
- [サービス依存関係マップ](../../../architecture/overview/サービス依存関係マップ.md)
- [共通: レイテンシ高騰 Runbook](../common/high-latency.md)
- [共通: サービスダウン Runbook](../common/service-down.md)
