# infra/observability — LGTM 観測スタック

ADR-OBS-001 / ADR-OBS-002 に従い、ログ / トレース / メトリクス / プロファイリングを
Grafana の LGTM スタックで統合観測する。本ディレクトリは production-grade defaults を持ち、
Argo CD で `observability` namespace に展開される。

## 含まれる component

| ディレクトリ | 役割 | upstream chart | 主バックエンド |
|---|---|---|---|
| [`grafana/`](grafana/) | ダッシュボード / データソース統合 UI | `grafana/grafana` | (datasource: loki / tempo / mimir / pyroscope) |
| [`loki/`](loki/) | ログ集約 | `grafana/loki` | MinIO（S3 互換、`k1s0-events` バケット） |
| [`tempo/`](tempo/) | 分散トレース集約 | `grafana/tempo` | MinIO |
| [`mimir/`](mimir/) | メトリクス集約（Prometheus 互換、スケーラブル） | `grafana/mimir-distributed` | MinIO |
| [`pyroscope/`](pyroscope/) | 継続的プロファイリング（CPU / memory / heap） | `grafana/pyroscope` | MinIO |
| [`otel-collector/`](otel-collector/) | OTel パイプライン（receiver / processor / exporter） | `open-telemetry/opentelemetry-collector` | (送信先: loki / tempo / mimir / pyroscope) |

## 信号フロー

```
tier1/2/3 アプリ（OTel SDK）
   │
   ├─ logs   ──┐
   ├─ traces ──┤   OTel Collector（DaemonSet + Deployment 2 段）
   ├─ metrics ─┤        │
   └─ profiles ┘        │
                        ├─ logs    → Loki   ─┐
                        ├─ traces  → Tempo  ─┤
                        ├─ metrics → Mimir  ─┤  → Grafana（統合 UI）
                        └─ profile → Pyrosc ─┘
                        ↓
                        MinIO（S3 互換、長期保管）
```

## ローカル開発との関係

`tools/local-stack/manifests/90-observability/` には `values-grafana.yaml` /
`values-loki.yaml` / `values-tempo.yaml` の 3 ファイルが kind 単一ノードで動く構成
（Loki SingleBinary、Tempo single binary、Grafana NodePort）として配置されている。
本 `infra/observability/` はそれを **production-grade defaults**（HA / S3 backend /
ServiceMonitor / RBAC / mimir + pyroscope 追加）に正規化したもの。

| 観点 | local-stack（dev） | infra/observability（prod default） |
|---|---|---|
| Loki | SingleBinary 1 replica / filesystem | scalable 構成 / S3（MinIO）backend |
| Tempo | single binary / filesystem | distributed / S3 backend |
| Mimir | （なし） | distributed / S3 backend |
| Pyroscope | （なし） | distributed / S3 backend |
| OTel Collector | （tier アプリ側で送信、collector なし） | DaemonSet（agent）+ Deployment（gateway）の 2 段 |
| Grafana | NodePort + 平文 admin | ClusterIP（Istio Gateway 経由）+ OIDC（Keycloak 連携） |

## 配備フロー

`deploy/apps/application-sets/observability.yaml`（plan 06-XX 予定）の ApplicationSet が
本ディレクトリ配下を ref する形で Argo CD Application を作る。
6 component を 1 namespace（`observability`）に集約する。

## 関連設計

- [ADR-OBS-001](../../docs/02_構想設計/adr/ADR-OBS-001-grafana-lgtm.md) — LGTM スタック採用根拠
- [ADR-OBS-002](../../docs/02_構想設計/adr/ADR-OBS-002-otel-pipeline.md) — OTel パイプライン構成
- IMP-OBS-* — Observability 実装ガイド
