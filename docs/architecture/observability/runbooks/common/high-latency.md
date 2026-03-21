# アラート: レイテンシ高騰

対象アラート: `SystemServiceHighLatency`, `ServiceHighLatency`,
`GrpcHighErrorRate`, `auth_server_high_latency`, `config_server_high_latency`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | warning |
| **影響範囲** | 対象サービスのレスポンスタイム |
| **通知チャネル** | Microsoft Teams #alert-warning |
| **対応 SLA** | SEV3（翌営業日） / 長時間継続なら SEV2（30分以内） |

## アラート発火条件

| アラート名 | Tier | 閾値 |
|-----------|------|------|
| SystemServiceHighLatency | system | P99 レイテンシ > 500ms で 5分継続 |
| ServiceHighLatency | business/service | P99 レイテンシ > 1s で 5分継続 |
| GrpcHighErrorRate | business/service | gRPC エラー率 > 5% で 5分継続 |

SLO 目標値: system Tier P99 < 200ms、business/service Tier P99 < 500ms/1s

## 初動対応（5分以内）

### 1. 現在のレイテンシ状況を確認

Grafana → サービス詳細ダッシュボード → Latency（P50/P95/P99）パネルを確認:
- [サービス詳細ダッシュボード](http://grafana.k1s0.internal/d/k1s0-service/k1s0-service-dashboard)

```bash
# Prometheus でリアルタイム確認
# histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{service="{service-name}"}[5m]))
```

### 2. 影響の深刻度を判断

- [ ] P99 > 5s で 10分継続 → SEV2（詳細調査を即時開始）
- [ ] SLO バーンレートアラートも同時発火 → SEV1 へエスカレーション検討
- [ ] 軽微な上昇かつ回復傾向あり → SEV3（監視継続）

### 3. 依存サービスのレイテンシを確認

```bash
# サービスメッシュ経由のレイテンシ確認（Kiali）
# http://kiali.k1s0.internal → Graph → {service-name} の依存関係を確認
```

## 詳細調査

### よくある原因

1. **DB クエリの遅延**: インデックス未使用・ロック競合・スロークエリ
2. **外部 API の遅延**: 依存するサービス・API のレスポンス低下
3. **リソース競合**: CPU スロットリング・メモリ圧迫による GC 増加
4. **コールドスタート**: Pod 再起動直後のウォームアップ期間

### 調査コマンド

```bash
# CPU スロットリングの確認
kubectl top pods -n {namespace} | grep {service-name}

# トレースで遅いスパンを特定（Jaeger）
# http://jaeger.k1s0.internal → Search → Service: {service-name} → Min Duration: 1s

# DB スロークエリログの確認
kubectl logs -n {namespace} deploy/{db-service} | grep "slow query"
```

### Prometheus クエリ例

```promql
# エンドポイント別 P99 レイテンシ
histogram_quantile(0.99,
  sum by (path, le) (
    rate(http_request_duration_seconds_bucket{service="{service-name}"}[5m])
  )
)

# gRPC メソッド別エラー率
sum by (grpc_method) (
  rate(grpc_server_handled_total{service="{service-name}", grpc_code!="OK"}[5m])
) / sum by (grpc_method) (
  rate(grpc_server_handled_total{service="{service-name}"}[5m])
)
```

## 復旧手順

### パターン A: DB 起因の場合

```bash
# DB の接続状況確認
kubectl exec -n {namespace} deploy/{service-name} -- \
  psql $DATABASE_URL -c "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"

# スロークエリの特定と対処（DBA へ連絡）
```

### パターン B: CPU スロットリングの場合

```bash
# リソース制限の一時的な緩和（Helm values 変更後再デプロイ）
kubectl patch deployment {service-name} -n {namespace} \
  --patch '{"spec":{"template":{"spec":{"containers":[{"name":"{service-name}","resources":{"limits":{"cpu":"2"}}}]}}}}'
```

### パターン C: Pod の再起動

```bash
# レイテンシが異常に高い特定 Pod の削除（再作成）
kubectl delete pod -n {namespace} {pod-name}
```

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] P99 > 5s で 30分以上継続
- [ ] SLO バーンレートアラートも同時発火
- [ ] 原因特定ができず改善の見通しが立たない
- [ ] 複数サービスで同時にレイテンシが上昇

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- Jaeger でトレースを確認し、どのスパンが遅いか特定する
- デプロイや設定変更との相関を調べる
- 時間帯・リクエスト量との相関（トラフィック増による劣化かどうか）

## 関連ドキュメント

- [トレーシング設計](../../トレーシング設計.md)
- [SLO 設計](../../SLO設計.md)
