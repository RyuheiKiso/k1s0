# 可観測性 - SLO 設計

D-108: SLO/SLA 定義。SLI、SLO、SLA、エラーバジェット運用を定義する。

元ドキュメント: [可観測性設計.md](./可観測性設計.md)

---

## D-108: SLO/SLA 定義

### SLI（Service Level Indicators）

| SLI          | 定義                                                    | 計測方法（HTTP）                            | 計測方法（gRPC）                             |
| ------------ | ------------------------------------------------------- | ------------------------------------------- | -------------------------------------------- |
| 可用性       | 正常レスポンス数 / 全リクエスト数                       | `http_requests_total{status!~"5.."} / total` | `grpc_server_handled_total{grpc_code="OK"} / total` |
| レイテンシ   | P99 レスポンスタイム                                    | `histogram_quantile(0.99, http_request_duration_seconds_bucket)` | `histogram_quantile(0.99, grpc_server_handling_seconds_bucket)` |
| エラーレート | 5xx / 非 OK レスポンス率                                | `rate(http_requests_total{status=~"5.."})`  | `rate(grpc_server_handled_total{grpc_code!="OK"})` |

### SLO（Service Level Objectives）

> **可用性とエラーレートの関係**: 可用性 SLO は「正常レスポンス数 / 全リクエスト数」で計測するため、可用性 99.95% はエラーレート < 0.05% と同義である。同様に、可用性 99.9% はエラーレート < 0.1% と同義である。以下の表では両指標を並記し、計測方法の違いを明示する。

#### system Tier

| SLO               | 目標値   | 計測期間 | エラーバジェット（30日） |
| ------------------ | -------- | -------- | ------------------------ |
| 可用性             | 99.95%   | 30 日    | 21.6 分                  |
| P99 レイテンシ     | < 200ms  | 30 日    | -                        |
| エラーレート       | < 0.05%  | 30 日    | -（可用性 99.95% と同義）|

#### business Tier

| SLO               | 目標値   | 計測期間 | エラーバジェット（30日） |
| ------------------ | -------- | -------- | ------------------------ |
| 可用性             | 99.9%    | 30 日    | 43.2 分                  |
| P99 レイテンシ     | < 500ms  | 30 日    | -                        |
| エラーレート       | < 0.1%   | 30 日    | -（可用性 99.9% と同義） |

#### service Tier

| SLO               | 目標値   | 計測期間 | エラーバジェット（30日） |
| ------------------ | -------- | -------- | ------------------------ |
| 可用性             | 99.9%    | 30 日    | 43.2 分                  |
| P99 レイテンシ     | < 1s     | 30 日    | -                        |
| エラーレート       | < 0.1%   | 30 日    | -（可用性 99.9% と同義） |

### SLA（Service Level Agreements）

#### 内部 SLA（チーム間合意）

| Tier     | 可用性    | P99 レイテンシ | 計測期間 |
| -------- | --------- | -------------- | -------- |
| system   | 99.9%     | < 500ms        | 月間     |
| business | 99.8%     | < 1s           | 月間     |
| service  | 99.8%     | < 2s           | 月間     |

#### SLA 違反時のエスカレーション

| 条件                   | アクション                                                     |
| ---------------------- | -------------------------------------------------------------- |
| リアルタイム可用性低下 | 即座にオンコール担当に通知（Alertmanager → Teams 連携）        |
| 月間 SLA 未達          | 月次レビューで原因分析と再発防止策を検討、ポストモーテム実施   |

### エラーバジェット運用

```
エラーバジェット = 1 - SLO目標値
エラーバジェット残量 = エラーバジェット - 実測エラー率
```

| バジェット残量 | アクション                                      |
| -------------- | ----------------------------------------------- |
| > 50%          | 通常運用。新機能リリース可能                    |
| 25% - 50%      | 注意。リリース頻度を下げ、信頼性改善に注力      |
| < 25%          | 警告。新機能リリースを凍結し、信頼性改善に専念  |
| 0%             | リリース凍結。ポストモーテム実施                |

#### バーンレートアラート

エラーバジェットの消費速度（バーンレート）に基づいてアラートを発火する。Google SRE の Multi-window, Multi-burn-rate アプローチを採用し、短期的な急激劣化と長期的な緩やかな劣化の両方を検知する。

> **バーンレートとは**: エラーバジェットの消費速度の倍率。バーンレート 1x はちょうど 30 日でバジェットを使い切る速度。14.4x は約 2 日で使い切る速度を意味する。

| アラート名 | バーンレート閾値 | 長時間窓 | 短時間窓 | severity | 対応速度 |
| --- | --- | --- | --- | --- | --- |
| SLOBurnRateCritical | 14.4x | 1h | 5m | critical | 即時対応（ページ） |
| SLOBurnRateWarning | 6x | 6h | 30m | warning | 計画対応（チケット） |

##### PromQL 式

```yaml
# system Tier (SLO 99.95%, error_budget = 0.0005)
- alert: SLOBurnRateCritical
  expr: |
    (
      (1 - (sum(rate(http_requests_total{namespace="k1s0-system", status!~"5.."}[1h])) by (service)
       / sum(rate(http_requests_total{namespace="k1s0-system"}[1h])) by (service)))
      / 0.0005
    ) > 14.4
    and
    (
      (1 - (sum(rate(http_requests_total{namespace="k1s0-system", status!~"5.."}[5m])) by (service)
       / sum(rate(http_requests_total{namespace="k1s0-system"}[5m])) by (service)))
      / 0.0005
    ) > 14.4
  for: 2m
  labels:
    severity: critical
    tier: system
  annotations:
    summary: "System Tier SLO burn rate critical: {{ $labels.service }}"
    description: "エラーバジェットの消費速度が 14.4 倍を超えています。約 2 日でバジェットを使い切ります。"

- alert: SLOBurnRateWarning
  expr: |
    (
      (1 - (sum(rate(http_requests_total{namespace="k1s0-system", status!~"5.."}[6h])) by (service)
       / sum(rate(http_requests_total{namespace="k1s0-system"}[6h])) by (service)))
      / 0.0005
    ) > 6
    and
    (
      (1 - (sum(rate(http_requests_total{namespace="k1s0-system", status!~"5.."}[30m])) by (service)
       / sum(rate(http_requests_total{namespace="k1s0-system"}[30m])) by (service)))
      / 0.0005
    ) > 6
  for: 5m
  labels:
    severity: warning
    tier: system

# business / service Tier (SLO 99.9%, error_budget = 0.001)
# 同構造で error_budget を 0.001 に変更し、namespace を k1s0-business|k1s0-service に変更
```

アラートルールのファイル配置は `infra/observability/prometheus/alerts/slo-burn-rate-alerts.yaml` とする。ローカル開発環境では Prometheus UI でバーンレートを確認する。

#### Prometheus Recording Rule

Recording Rules の定義は [可観測性設計.md](./可観測性設計.md) の「Recording Rules」セクションを参照。SLO 関連の Recording Rules（`slo:availability:ratio`、`slo:error_budget:remaining`）は、RED メトリクスの Recording Rules と合わせて `infra/docker/prometheus/recording_rules.yaml` で一元管理する。

---

## 関連ドキュメント

- [可観測性設計.md](./可観測性設計.md) -- 基本方針・概要
- [可観測性-監視アラート設計.md](./監視アラート設計.md) -- 監視・アラート設計
- [可観測性-トレーシング設計.md](./トレーシング設計.md) -- 分散トレーシング
- [可観測性-ログ設計.md](./ログ設計.md) -- 構造化ログ・Loki
