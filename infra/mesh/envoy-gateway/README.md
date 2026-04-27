# infra/mesh/envoy-gateway — Envoy Gateway（南北 ingress）

ADR-CNCF-004（Envoy Gateway 選定）/ ADR-DIR-002（infra 分離）/ IMP-DIR-INFRA-073（サービスメッシュ配置）に従い、
クラスタ外部からの L7 ingress を Gateway API 標準準拠で運用する。Istio Ambient（東西 mTLS）と
責務を分離し、Envoy Gateway は南北のみを担う。

## なぜ Envoy Gateway か

- **Gateway API 標準準拠**: Kubernetes 標準の `gateway.networking.k8s.io/v1` を採用するため、Ingress 層の将来移行コストが低い。
- **Istio Ingress Gateway を使わない**: Ambient mode は east-west（pod ↔ pod）に特化する設計で、
  ingress は別 component に分離した方が運用上の責務が明瞭。
- **拡張性**: EnvoyPatchPolicy / Backend / SecurityPolicy 等の Envoy Gateway 固有 CRD で
  L7 ポリシーを宣言的に追加できる。

## ファイル構成

| ファイル | 内容 |
|---|---|
| `values.yaml` | Envoy Gateway controller の Helm values（HA 3 replica + OTel tracing 連携 + Prometheus メトリクス） |
| `gatewayclass-envoy.yaml` | GatewayClass `envoy-gateway-class` 定義（全 Gateway の親） |
| `gateway-public.yaml` | インターネット公開用 Gateway（`api.k1s0.internal` / `app.k1s0.internal`、MetalLB public プール経由） |
| `gateway-internal.yaml` | VPN / Bastion 内部公開用 Gateway（`ops.k1s0.internal` / `portal.k1s0.internal`、MetalLB internal プール経由） |
| `httproute/tier1-api.yaml` | tier1 公開 12 API への path 単位ルーティング（`/api/v1/{state,pubsub,...}` → t1-state / t1-secret / t1-workflow / Rust 3 Pod） |
| `httproute/tier3-web.yaml` | tier3 Web SPA / BFF へのルーティング（`/admin` `/docs` `/` + `/api/portal` `/api/admin`） |
| `httproute/redirect-http-to-https.yaml` | HTTP → HTTPS の 301 恒久リダイレクト |

## デプロイ

```sh
# Envoy Gateway controller の install（Argo CD ApplicationSet からも可）
helm install eg oci://docker.io/envoyproxy/gateway-helm \
  --namespace envoy-gateway-system --create-namespace \
  --version v1.2.0 -f infra/mesh/envoy-gateway/values.yaml

# GatewayClass / Gateway / HTTPRoute を順に apply
kubectl apply -f infra/mesh/envoy-gateway/gatewayclass-envoy.yaml
kubectl apply -f infra/mesh/envoy-gateway/gateway-public.yaml
kubectl apply -f infra/mesh/envoy-gateway/gateway-internal.yaml
kubectl apply -f infra/mesh/envoy-gateway/httproute/
```

prod では Argo CD ApplicationSet `tier-mesh`（plan 06-XX）から本ディレクトリを kustomize で適用する。

## Istio Ambient との責務分離

| レイヤ | 担当 component | 通信種別 | 暗号化 |
|---|---|---|---|
| 南北（外部 ↔ クラスタ） | **Envoy Gateway**（本ディレクトリ） | HTTPS / gRPC | TLS 終端（cert-manager） |
| 東西（pod ↔ pod） | **Istio Ambient**（`infra/mesh/istio-ambient/`） | L4 / L7 | ztunnel mTLS STRICT |
| 東西の L7 認可 | Istio Waypoint Proxy | L7 | （ztunnel 上） |

南北リクエストの典型フロー:

1. クライアント → MetalLB external IP → Envoy Gateway controller（TLS 終端、HTTPRoute 評価）
2. Envoy Gateway → backend Service（pod IP）
3. ztunnel が pod-to-pod 通信を mTLS で再暗号化
4. backend pod の Dapr sidecar / アプリ コンテナへ到達

## ローカル開発との差分

| 観点 | dev（`tools/local-stack/manifests/30-envoy-gateway/`、暫定） | prod（本ディレクトリ） |
|---|---|---|
| Envoy Gateway controller replica | 1 | 3 + Pod AntiAffinity |
| ホスト名 | `*.k1s0.local`（/etc/hosts） | `*.k1s0.internal`（社内 DNS） |
| TLS | self-signed | cert-manager（letsencrypt-prod / internal-ca） |
| OTel tracing | （無効） | OTel Collector gateway へ送信、sampling 100% |
| Prometheus | （無効） | ServiceMonitor 有効、port 19001 |

## 関連設計

- [ADR-CNCF-004](../../../docs/02_構想設計/adr/) — Envoy Gateway 選定
- [ADR-MIG-002](../../../docs/02_構想設計/adr/ADR-MIG-002-istio-ambient-migration.md) — sidecar → ambient 移行
- [ADR-OBS-002](../../../docs/02_構想設計/adr/ADR-OBS-002-otel-pipeline.md) — Envoy Gateway から OTel exporter
- [IMP-DIR-INFRA-073](../../../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/03_サービスメッシュ配置.md)
