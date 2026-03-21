# アラート: SLO バーンレート Critical

対象アラート: `SLOBurnRateCritical`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical |
| **影響範囲** | 対象サービスの SLO エラーバジェット（2日以内に枯渇する速度） |
| **通知チャネル** | Microsoft Teams #alert-critical |
| **対応 SLA** | SEV1（15分以内） |

## アラート発火条件

Multi-window, Multi-burn-rate アラート:
- **バーンレート > 14.4** かつ
  - 1時間窓 と 5分窓 の両方でバーンレートが閾値超過
- 意味: このペースが続くと **2日以内にエラーバジェットが枯渇**

## SLO / エラーバジェット参考値

| Tier | SLO | エラーバジェット (30日) |
|------|-----|---------------------|
| system | 99.95% | 21.6 分 |
| business/service | 99.9% | 43.2 分 |

## 初動対応（5分以内）

### 1. エラーバジェットの残量確認

Grafana → SLO ダッシュボードを確認:
- [SLO ダッシュボード](http://grafana.k1s0.internal/d/k1s0-slo/slo-dashboard)

```promql
# 残エラーバジェット（%）
1 - (
  1 - (sum(rate(http_requests_total{namespace="k1s0-system", status!~"5.."}[30d])) by (service)
  / sum(rate(http_requests_total{namespace="k1s0-system"}[30d])) by (service))
) / 0.0005
```

### 2. エラー発生源の特定

```bash
# エラー率を確認
# rate(http_requests_total{namespace="k1s0-system", status=~"5.."}[5m])
# / rate(http_requests_total{namespace="k1s0-system"}[5m])
```

→ エラー率が高い場合は [high-error-rate.md](./high-error-rate.md) も参照。

### 3. 即時判断（SEV1 として扱う）

SLOBurnRateCritical は原則 SEV1。直ちに以下を実施:
- [ ] 影響範囲の特定（エンドポイント・ユーザー数）
- [ ] インシデントチャネルを作成
- [ ] 関係者に通知

## 詳細調査

### バーンレートの算出根拠

バーンレート 14.4x は以下を意味する:
- 通常の 14.4 倍のペースでエラーバジェットを消費中
- 2日（48時間）でエラーバジェットが枯渇

### 調査手順

```bash
# 1. エラー率の確認（高エラー率と組み合わさっている可能性）
# → high-error-rate.md の手順を実施

# 2. デプロイ履歴の確認
kubectl rollout history deployment/{service-name} -n {namespace}

# 3. トレースで根本原因を特定
# Jaeger UI → http://jaeger.k1s0.internal
# Service: {service-name} → Tags: error=true → 最新のエラートレース
```

## 復旧手順

### ステップ 1: エラー源の停止

エラー率が高い場合、エラー源の特定と停止が最優先:
```bash
# 問題のあるデプロイのロールバック
kubectl rollout undo deployment/{service-name} -n {namespace}
```

### ステップ 2: エラーバジェットの回復確認

```bash
# バーンレートが 14.4 未満になったことを Prometheus で確認
# SLOBurnRateCritical アラートが resolve されるまで監視継続
```

### ステップ 3: ポストモーテムの実施

SLOBurnRateCritical が発火した場合、復旧後にポストモーテムを実施する:
- テンプレート: [../postmortem-template.md](../postmortem-template.md)

## エスカレーション基準

SLOBurnRateCritical は自動的に SEV1 として扱う。以下の場合はさらに上位へ:

- [ ] エラーバジェットが 50% 未満まで消費されている
- [ ] 複数サービスで同時に critical バーンレート
- [ ] ロールバック後もバーンレートが改善しない

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- バーンレートが上昇し始めた時点（Prometheus の履歴）と直近のデプロイの相関
- エラーバジェットの消費パターン（特定時間帯・特定エンドポイントへの集中）
- 再発防止策として SLO アラートの閾値見直しが必要かどうか

## 関連ドキュメント

- [SLO 設計](../../SLO設計.md)
- [slo-burn-rate-warning.md](./slo-burn-rate-warning.md)
- [high-error-rate.md](./high-error-rate.md)
