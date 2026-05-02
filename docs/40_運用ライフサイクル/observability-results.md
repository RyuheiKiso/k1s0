# 観測性 E2E 結果サマリ（月次更新）

本書は ADR-TEST-006（観測性 E2E を 5 検証で構造化）の月次実走結果を時系列で記録する live document。Grafana LGTM スタック（ADR-OBS-001 / 002）が機能継続検証されていることを採用検討組織に公開する。

## 月次サマリ

### 2026-05（リリース時点 / 初月、初回 local 実走 — 1/4 検証 PASS）

- **状態**: kind cluster + Grafana LGTM スタック（Prometheus / Loki / Tempo / OTel Collector）起動済の環境で観測性 E2E 4 件中 1 件 PASS（2026-05-03 00:11 JST 実走）
- **PASS**: TestPrometheusCardinality（`/api/v1/labels` で 102 label name 取得 / `up` metric cardinality=10）
- **FAIL（採用初期で対応）**:
  - TestLokiLogTraceCorrelation: `/loki/api/v1/labels: data 空` — Loki が log を一切受信していない。promtail / fluentd 等の log shipping が observability layer に含まれていない。採用初期で `tools/local-stack/up.sh` の observability layer に promtail を追加する経路を整備
  - TestOTLPTracePropagation: `ForceFlush: context deadline exceeded` — port-forward 経由の OTLP gRPC 接続で BatchSpanProcessor が flush できない。OTel Collector の OTLP gRPC receiver は localhost:4317 で listen 確認済だが、port-forward の TCP 経路に問題。採用初期で in-cluster job 経由の trace 送信に切替検討
- **Skip**: TestSLOAlertManagerEndpoint — Alertmanager が observability layer に含まれていないため起動なし。kube-prometheus-stack を別 layer として追加で対応
- **環境構築経路**:
  - `tools/local-stack/up.sh --no-cluster --layers observability --observability --role tier1-go-dev`
  - 起動時に PVC が「no persistent volumes available」で Pending → standard StorageClass に default annotation を patch（root fix: `kubectl patch storageclass standard -p '{"metadata":{"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'`）で解決
  - 4 並列 port-forward: otel-collector→4317 / tempo→3200 / prometheus→9090 / loki→3100
- **以降**: log shipping（promtail）整備 / OTLP 接続経路改善 / Alertmanager 追加 / dashboard JSON 整備（検証 5 のため）

## 月次サマリ template

```markdown
### YYYY-MM

- **対象期間**: YYYY-MM-01 〜 YYYY-MM-末日
- **5 検証中 PASS 数**: N/5
- **検証 1（trace 貫通）**: PASS / FAIL（root cause）
- **検証 2（cardinality）**: PASS / FAIL（baseline diff）
- **検証 3（log↔trace）**: PASS / FAIL（結合率 X.X%）
- **検証 4（SLO alert）**: PASS / FAIL（fast burn 発火秒数）
- **検証 5（dashboard）**: PASS / FAIL（baseline diff 件数）
```

## 関連

- ADR-TEST-006（観測性 E2E 5 検証）
- ADR-OBS-001（Grafana LGTM）/ ADR-OBS-002（OTel Collector）/ ADR-OBS-003（Incident Taxonomy）
- ADR-OPS-001（Runbook `runbook_url` 必須要件と検証 4 の連携）
- `tests/e2e/observability/` 配下の 4 検証
