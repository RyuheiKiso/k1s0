# アラート名: bff-proxy サービス障害

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical |
| **影響範囲** | 全クライアント（モバイル・Web アプリ）から全 API へのアクセス |
| **通知チャネル** | Microsoft Teams #alert-critical |
| **対応 SLA** | SEV1: 15分以内 |

## アラート発火条件

```promql
# サービスダウン
up{job="bff-proxy"} == 0

# エラーレート高騰（BFF は全トラフィックの入口なため閾値を低めに設定）
sum(rate(http_requests_total{job="bff-proxy",status=~"5.."}[5m]))
  / sum(rate(http_requests_total{job="bff-proxy"}[5m])) > 0.02

# Kong ルーティング失敗
rate(kong_http_requests_total{code="502"}[5m]) > 5

# P99 レイテンシ超過
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{job="bff-proxy"}[5m])) > 2.0
```

## 初動対応（5分以内）

### 1. 状況確認

```bash
# Pod の状態確認
kubectl get pods -n k1s0-system -l app=bff-proxy

# Kong の状態確認（bff-proxy は Kong 経由でルーティング）
kubectl get pods -n k1s0-system -l app=kong

# Kong の管理 API でルーティング状態を確認する
kubectl exec -n k1s0-system deploy/kong -- \
  curl -s http://127.0.0.1:8001/status

# 直近のエラーログ確認
# Grafana → Explore → Loki: {app="bff-proxy", level="error"} | 直近10分

# アップストリームサービスの状態確認
kubectl get pods -n k1s0-system
kubectl get pods -n k1s0-service
```

### 2. Grafana ダッシュボード確認

- [bff-proxy ダッシュボード](http://grafana.k1s0.internal/d/k1s0-bff/bff-dashboard)
- [Kong ダッシュボード](http://grafana.k1s0.internal/d/k1s0-kong/kong-dashboard)
- [SLO ダッシュボード](http://grafana.k1s0.internal/d/k1s0-slo/slo-dashboard)

### 3. 即時判断

- [ ] bff-proxy Pod が全て落ちている → SEV1（即時エスカレーション）
- [ ] Kong が応答しない → SEV1（Kong 障害）
- [ ] 特定のルートのみエラー → SEV2（アップストリームサービス障害）
- [ ] レイテンシのみ悪化（エラーなし） → SEV2（パフォーマンス劣化）

## 詳細調査

### よくある原因

1. **Kong ルーティング設定の誤り**: `kong-sync.yaml` 適用後に設定が壊れた
2. **アップストリームサービスの障害**: バックエンドサービスが応答せず 502/503 が返る
3. **レート制限設定の誤り**: Kong プラグインの設定が厳しすぎて正常トラフィックが拒否される
4. **OOM Kill**: リクエスト集中によるメモリ不足
5. **mTLS 証明書の期限切れ**: Istio サイドカーの証明書が期限切れでサービス間通信が失敗

### 調査コマンド

```bash
# Kong のルート一覧を確認する
kubectl exec -n k1s0-system deploy/kong -- \
  curl -s http://127.0.0.1:8001/routes | jq '.data[].name'

# 特定ルートの設定を確認する
kubectl exec -n k1s0-system deploy/kong -- \
  curl -s "http://127.0.0.1:8001/routes/<route-name>"

# Kong のアップストリームヘルスチェック
kubectl exec -n k1s0-system deploy/kong -- \
  curl -s "http://127.0.0.1:8001/upstreams/<upstream-name>/health"

# Istio サイドカーの証明書確認
kubectl exec -n k1s0-system deploy/bff-proxy -c istio-proxy -- \
  pilot-agent request GET /certs | jq '.certs[0].expiry'

# レート制限カウンターの確認（Redis）
kubectl exec -n k1s0-system deploy/redis -- \
  redis-cli -h redis.k1s0-system.svc.cluster.local \
    --scan --pattern "kong:rate_limit:*" | head -20
```

## 復旧手順

### パターン A: bff-proxy Pod 障害の場合

```bash
kubectl rollout restart deployment/bff-proxy -n k1s0-system
kubectl rollout status deployment/bff-proxy -n k1s0-system
```

### パターン B: Kong 設定の問題の場合

```bash
# 直前の Kong 設定をロールバックする
# kong-sync.yaml の前バージョンのコミットを特定する
git log --oneline .github/workflows/kong-sync.yaml

# 前バージョンの Kong 設定を手動で適用する
kubectl exec -n k1s0-system deploy/kong -- \
  curl -X DELETE "http://127.0.0.1:8001/routes/<broken-route>"

# または Kong 設定をリセットして再適用する
# infra/kong/ の設定ファイルを確認して正しい状態に戻す
```

### パターン C: アップストリームサービス障害の場合

```bash
# 問題のアップストリームを特定する
kubectl exec -n k1s0-system deploy/kong -- \
  curl -s http://127.0.0.1:8001/upstreams | jq '.data[].name'

# 各アップストリームのヘルス確認
kubectl exec -n k1s0-system deploy/kong -- \
  curl -s "http://127.0.0.1:8001/upstreams/<upstream>/health" | jq '.data[].health'

# 障害のあるアップストリームサービスの Runbook を参照して復旧する
```

### パターン D: mTLS 証明書期限切れの場合

```bash
# Istio Pilot に証明書のローテーションを要求する
kubectl rollout restart deployment/bff-proxy -n k1s0-system

# cert-manager の証明書状態を確認する
kubectl get certificate -n k1s0-system
kubectl describe certificate -n k1s0-system <cert-name>
```

## エスカレーション基準

- [ ] bff-proxy と Kong が同時に障害になっている
- [ ] 全クライアントからのアクセスが完全に遮断されている
- [ ] 証明書期限切れが原因で cert-manager チームへの連絡が必要
- [ ] 15分以内に復旧の見通しが立たない

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md) のエスカレーションパスを参照。

## 根本原因分析のポイント

- Kong 設定変更後の検証プロセス（kong-sync.yaml のロールバック手順）を整備する
- Istio 証明書ローテーションの自動化が正常に動作しているか確認する
- アップストリームのヘルスチェック間隔とタイムアウト設定を見直す

## 関連ドキュメント

- [ゲートウェイ責務分離](../../../infrastructure/overview/ゲートウェイ責務分離.md)
- [certificate-expiring Runbook](../common/certificate-expiring.md)
- [デプロイ手順書](../../../infrastructure/kubernetes/デプロイ手順書.md)
- [インシデント管理設計](../インシデント管理設計.md)
