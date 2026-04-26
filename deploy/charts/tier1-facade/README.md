# `tier1-facade` Helm chart

k1s0 の tier1 Go ファサード（state / secret / workflow の 3 Pod）を Kubernetes に
配備する最小 Helm chart。

設計正典:

- [`docs/05_実装/70_リリース設計/`](../../../docs/05_実装/70_リリース設計/)（IMP-REL-*）
- [`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md`](../../../docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md)
- ADR-CICD-001（Argo CD）/ ADR-CICD-002（Argo Rollouts）/ ADR-TIER1-001

## 提供範囲

| リソース | 個数 | 説明 |
|---|---|---|
| `Deployment` | 3（state / secret / workflow） | 各 Pod 独立、distroless / nonroot / readOnlyRootFilesystem |
| `Service` | 3 | gRPC 50001 + metrics 9090、`appProtocol: grpc` |
| `ConfigMap` | 1（共通） | OTel endpoint / log level |
| `ServiceAccount` | 1（共通） | SPIRE 連携用 annotation 注入点 |

## scope

- **リリース時点**（本 chart）: 通常 `Deployment` での 3 Pod 配備、gRPC liveness/readiness probe、Prometheus scrape annotation
- **採用初期**: Argo Rollouts 移行（canary 戦略 + AnalysisTemplate）、HPA / KEDA、SPIRE workload identity（`spiffe.io/spiffe-id` annotation）、Istio Ambient L7 waypoint
- **採用後の運用拡大時**: マルチクラスタフェデレーション、cross-cluster mTLS

## 利用例

```bash
# install（namespace 作成 + 配備）
helm install tier1-facade deploy/charts/tier1-facade \
    --namespace k1s0-tier1 \
    --create-namespace \
    --set image.tag=v0.1.0

# upgrade
helm upgrade tier1-facade deploy/charts/tier1-facade \
    --namespace k1s0-tier1 \
    --set image.tag=v0.2.0

# uninstall
helm uninstall tier1-facade --namespace k1s0-tier1
```

## values.yaml の主要パラメータ

| キー | 既定値 | 説明 |
|---|---|---|
| `image.registry` | `ghcr.io` | container registry |
| `image.repositoryPrefix` | `k1s0/k1s0` | repository prefix（`tier1-state` 等が後段で付く） |
| `image.tag` | `latest` | image tag（CI が SemVer + commit SHA で push） |
| `pods.<name>.enabled` | `true` | 当該 Pod を配備するか |
| `pods.<name>.replicas` | `1` | replicas（production は HPA/KEDA で 2+ 推奨） |
| `pods.<name>.resources` | 50m/64Mi req, 500m/256Mi limit | 起動下限（採用初期で実測に置換） |
| `config.otelExporterEndpoint` | `otel-collector.observability.svc.cluster.local:4317` | OTel collector |
| `service.type` | `ClusterIP` | Istio Ambient ztunnel 経由で十分 |

## 検証方法（lint / template）

```bash
# シンタックス検証
helm lint deploy/charts/tier1-facade

# 出力 manifest を確認
helm template tier1-facade deploy/charts/tier1-facade \
    --namespace k1s0-tier1 \
    --set image.tag=v0.1.0
```

## 関連 chart（雛形のみ、採用初期で実装）

- `deploy/charts/tier1-rust-service/` — tier1 Rust 自作領域 chart
- `deploy/charts/tier2-{go,dotnet}-service/` — tier2 Pod 別 chart
- `deploy/charts/tier3-{web-app,bff}/` — tier3 Pod 別 chart
