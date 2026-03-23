# モニタリングカバレッジ

## 概要

本ドキュメントは k1s0-system 内の全サービスに対する ServiceMonitor および PrometheusRule の設定状況を一覧化したものです。
外部監査指摘 H-02（監視・アラートカバレッジ不足）への対応として、全主要サービスへの監視設定を整備しました。

## ServiceMonitor 設定状況

ServiceMonitor は Prometheus Operator が各サービスのメトリクスエンドポイントを自動検出するための設定です。
全サービスに対して 15 秒間隔でスクレイピングを行います。

| サービス | ファイルパス | 設定状況 |
|---|---|---|
| auth | `infra/observability/auth/servicemonitor.yaml` | 設定済み |
| config | `infra/observability/config/servicemonitor.yaml` | 設定済み |
| saga | `infra/observability/saga/servicemonitor.yaml` | 設定済み |
| bff-proxy | `infra/observability/bff-proxy/servicemonitor.yaml` | 設定済み |
| dlq-manager | `infra/observability/dlq-manager/servicemonitor.yaml` | 設定済み |
| session | `infra/observability/session/servicemonitor.yaml` | 設定済み（H-02 対応）|
| tenant | `infra/observability/tenant/servicemonitor.yaml` | 設定済み（H-02 対応）|
| workflow | `infra/observability/workflow/servicemonitor.yaml` | 設定済み（H-02 対応）|
| notification | `infra/observability/notification/servicemonitor.yaml` | 設定済み（H-02 対応）|
| vault | `infra/observability/vault/servicemonitor.yaml` | 設定済み（H-02 対応）|

## PrometheusRule 設定状況

PrometheusRule は各サービスのアラートルールを定義します。
各サービスに以下の 3 種類のアラートを設定しています。

- **high_error_rate**: エラーレートが 5 分間 5% 超で `critical` アラート
- **high_latency**: P99 レイテンシが 5 分間 2 秒超で `warning` アラート
- **pod_restarts**: 15 分間に 3 回以上の Pod 再起動で `critical` アラート

| サービス | ファイルパス | 設定状況 |
|---|---|---|
| auth | `infra/observability/auth/prometheusrule.yaml` | 設定済み |
| config | `infra/observability/config/prometheusrule.yaml` | 設定済み |
| session | `infra/observability/session/prometheusrule.yaml` | 設定済み（H-02 対応）|
| saga | `infra/observability/saga/prometheusrule.yaml` | 設定済み（H-02 対応）|
| bff-proxy | `infra/observability/bff-proxy/prometheusrule.yaml` | 設定済み（H-02 対応）|
| tenant | `infra/observability/tenant/prometheusrule.yaml` | 設定済み（H-02 対応）|
| workflow | `infra/observability/workflow/prometheusrule.yaml` | 設定済み（H-02 対応）|
| dlq-manager | `infra/observability/dlq-manager/prometheusrule.yaml` | 設定済み（H-02 対応）|
| notification | `infra/observability/notification/prometheusrule.yaml` | 設定済み（H-02 対応）|

## インフラ共通アラート

サービス個別のルールに加え、インフラ全体を対象とした共通アラートも設定済みです。

| ファイルパス | 内容 |
|---|---|
| `infra/observability/prometheus/alerts/system-tier-alerts.yaml` | system tier 共通アラート |
| `infra/observability/prometheus/alerts/business-service-tier-alerts.yaml` | business/service tier 共通アラート |
| `infra/observability/prometheus/alerts/infrastructure-alerts.yaml` | インフラ（Kubernetes、Kafka 等）アラート |
| `infra/observability/prometheus/alerts/slo-burn-rate-alerts.yaml` | SLO バーンレートアラート |
| `infra/observability/prometheus/alerts/slo-recording-rules.yaml` | SLO 記録ルール |

## 変更履歴

| 日付 | 対応内容 |
|---|---|
| 2026-03-24 | 外部監査 H-02 対応: session, tenant, workflow, notification, vault の ServiceMonitor を追加 |
| 2026-03-24 | 外部監査 H-02 対応: session, saga, bff-proxy, tenant, workflow, dlq-manager, notification の PrometheusRule を追加 |
