# アラート: SLO バーンレート Warning

対象アラート: `SLOBurnRateWarning`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | warning |
| **影響範囲** | 対象サービスの SLO エラーバジェット（5日以内に枯渇する速度） |
| **通知チャネル** | Microsoft Teams #alert-warning |
| **対応 SLA** | SEV3（翌営業日） / バーンレートが上昇中なら SEV2（30分以内） |

## アラート発火条件

Multi-window, Multi-burn-rate アラート:
- **バーンレート > 6** かつ
  - 6時間窓 と 30分窓 の両方でバーンレートが閾値超過
- 意味: このペースが続くと **5日以内にエラーバジェットが枯渇**

## 初動対応（5分以内）

### 1. エラーバジェット残量とバーンレートの確認

Grafana → SLO ダッシュボードを確認:
- [SLO ダッシュボード](http://grafana.k1s0.internal/d/k1s0-slo/slo-dashboard)

### 2. バーンレートの傾向確認

```promql
# 現在のバーンレート（system Tier）
(
  1 - (sum(rate(http_requests_total{namespace="k1s0-system", status!~"5.."}[6h])) by (service)
  / sum(rate(http_requests_total{namespace="k1s0-system"}[6h])) by (service))
) / 0.0005
```

### 3. 判断基準

- [ ] バーンレートが 10 以上かつ上昇中 → SEV2（即日対応）
- [ ] バーンレートが 6〜10 で安定している → SEV3（翌営業日）
- [ ] `SLOBurnRateCritical` も同時発火 → [slo-burn-rate-critical.md](./slo-burn-rate-critical.md) へ

## 詳細調査

バーンレート 6x は以下を意味する:
- 通常の 6 倍のペースでエラーバジェットを消費中
- 5日でエラーバジェットが枯渇

```bash
# エラー率の確認
# rate(http_requests_total{status=~"5.."}[6h]) / rate(http_requests_total[6h])

# 最近のデプロイ確認
kubectl rollout history deployment/{service-name} -n {namespace}
```

## 復旧手順

### ステップ 1: エラー発生源を特定

→ [high-error-rate.md](./high-error-rate.md) の詳細調査を実施。

### ステップ 2: エラーバジェット回復の見通し確認

```promql
# エラーバジェット残量（%）
100 * (1 - (
  1 - (sum(rate(http_requests_total{namespace="k1s0-system", status!~"5.."}[30d])) by (service)
  / sum(rate(http_requests_total{namespace="k1s0-system"}[30d])) by (service))
) / 0.0005)
```

残量が 50% 未満の場合は優先度を上げて対処する。

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] バーンレートが 10 を超えて上昇し続けている
- [ ] `SLOBurnRateCritical` に昇格
- [ ] エラーバジェット残量が 25% 未満

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- 6時間窓でのエラー率推移（断続的なエラーか継続的なエラーか）
- エラーバジェットの消費パターンを月次レビューで確認

## 関連ドキュメント

- [SLO 設計](../../SLO設計.md)
- [slo-burn-rate-critical.md](./slo-burn-rate-critical.md)
