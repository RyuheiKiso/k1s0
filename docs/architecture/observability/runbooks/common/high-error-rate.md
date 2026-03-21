# アラート: エラー率高騰

対象アラート: `SystemServiceErrorRateWarning`, `SystemServiceHighErrorRate`,
`ServiceErrorRateWarning`, `ServiceHighErrorRate`, `auth_server_high_error_rate`, `config_server_high_error_rate`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical（エラー率 > 1%/5%） / warning（エラー率 > 0.1%/0.2%） |
| **影響範囲** | 対象サービスのリクエスト処理 |
| **通知チャネル** | Microsoft Teams #alert-critical / #alert-warning |
| **対応 SLA** | critical: SEV1（15分以内） / warning: SEV3（翌営業日） |

## アラート発火条件

| アラート名 | Tier | 閾値 |
|-----------|------|------|
| SystemServiceErrorRateWarning | system | 5xx エラー率 > 0.1% で 5分継続 |
| SystemServiceHighErrorRate | system | 5xx エラー率 > 1% で 5分継続 |
| ServiceErrorRateWarning | business/service | 5xx エラー率 > 0.2% で 5分継続 |
| ServiceHighErrorRate | business/service | 5xx エラー率 > 5% で 5分継続 |

## 初動対応（5分以内）

### 1. 影響範囲の特定

```bash
# エラーが発生しているサービスの確認
kubectl get pods -n k1s0-system
kubectl get pods -n k1s0-business
kubectl get pods -n k1s0-service

# Prometheus でエラー率を確認
# rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])
```

### 2. エラーログの確認（Loki）

Grafana → Explore → Loki で以下のクエリを実行:

```logql
# 直近5分のエラーログ
{namespace="k1s0-system", service="{service-name}"} |= "error" | json | level="error"

# スタックトレースの確認
{namespace="k1s0-system", service="{service-name}"} | json | level="error" | line_format "{{.message}}"
```

### 3. Grafana ダッシュボード確認

- [サービス概要ダッシュボード](http://grafana.k1s0.internal/d/k1s0-service/k1s0-service-dashboard) → Error Rate パネル
- [System Overview](http://grafana.k1s0.internal/d/system-overview/system-overview) → HTTP Error Rate

### 4. 即時判断

- [ ] エラー率 > 50% かつ回復傾向なし → SEV1（即時エスカレーション）
- [ ] 特定エンドポイントのみ → SEV2（詳細調査へ）
- [ ] 軽微で自動回復中 → SEV3（監視継続）

## 詳細調査

### よくある原因

1. **依存サービスの障害**: DB / Kafka / 外部 API の接続失敗
2. **デプロイ起因**: 直近のリリースによるバグ
3. **リソース枯渇**: メモリ不足、コネクションプール枯渇
4. **設定変更**: 環境変数・シークレットの変更ミス

### 調査コマンド

```bash
# 直近デプロイの確認
kubectl rollout history deployment/{service-name} -n {namespace}

# Pod のリソース使用状況
kubectl top pods -n {namespace} | grep {service-name}

# 直近のイベント確認
kubectl describe pod -n {namespace} -l app={service-name} | tail -20

# DB 接続確認
kubectl exec -n {namespace} deploy/{service-name} -- curl -sf http://localhost:8080/healthz
```

### Prometheus クエリ例

```promql
# エンドポイント別エラー率
sum by (path) (rate(http_requests_total{service="{service-name}", status=~"5.."}[5m]))
/ sum by (path) (rate(http_requests_total{service="{service-name}"}[5m]))

# エラーの種類別内訳
sum by (status) (rate(http_requests_total{service="{service-name}", status=~"5.."}[5m]))
```

## 復旧手順

### パターン A: デプロイ起因の場合

```bash
# 直前のバージョンにロールバック
kubectl rollout undo deployment/{service-name} -n {namespace}

# ロールバック状態の確認
kubectl rollout status deployment/{service-name} -n {namespace}
```

### パターン B: 依存サービスの障害の場合

```bash
# 依存サービスのヘルスチェック
kubectl get pods -n {dependency-namespace}

# 依存サービスの再起動（最終手段）
kubectl rollout restart deployment/{dependency-service} -n {dependency-namespace}
```

### パターン C: リソース枯渇の場合

```bash
# Pod の再起動
kubectl rollout restart deployment/{service-name} -n {namespace}

# 必要に応じてスケールアウト
kubectl scale deployment/{service-name} -n {namespace} --replicas={n}
```

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] エラー率 > 50% かつ 10分以上継続
- [ ] SLO バーンレートアラート（`SLOBurnRateCritical`）も同時発火
- [ ] ロールバック後も回復しない
- [ ] 複数サービスに連鎖障害が発生

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- デプロイタイミングとエラー発生タイミングの相関
- 依存サービスとの因果関係（トレース ID で Jaeger を確認）
- エラーの種類（500/502/503/504 の分類）
- リソースメトリクス（CPU/メモリ/コネクション数）の推移

## 関連ドキュメント

- [監視アラート設計](../../監視アラート設計.md)
- [SLO 設計](../../SLO設計.md)
- [トレーシング設計](../../トレーシング設計.md)
