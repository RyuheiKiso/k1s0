# infra/mesh/istio-ambient — Istio Ambient Mesh

ADR-MIG-002（sidecar → ambient 移行）/ ADR-SEC-002（mTLS STRICT）に従い、
Istio Ambient Mesh（CNI + ztunnel + 任意の waypoint proxy）を運用する。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | istioctl install 用 IstioOperator manifest（istiod HA 3 replica + HPA、OTel tracing 連携、ztunnel resource sizing） |
| `peer-authentication.yaml` | mesh 全体 mTLS STRICT 強制 |

## デプロイ

```sh
# CNI + ztunnel + istiod を ambient profile で install
istioctl install -f infra/mesh/istio-ambient/values.yaml -y

# mTLS STRICT 強制
kubectl apply -f infra/mesh/istio-ambient/peer-authentication.yaml
```

prod では Argo CD ApplicationSet（plan 06-XX）から `istioctl operator init` ベースで適用する。

## ローカル開発との差分

| 観点 | dev（`tools/local-stack/manifests/30-istio-ambient/`） | prod（本ディレクトリ） |
|---|---|---|
| istiod replica | 1 | 3 + HPA（CPU 70%、3〜10） |
| tracing | enableTracing のみ | OTel Collector gateway へ送信、sampling 1.0 |
| access log | （無効） | JSON で stdout |
| mTLS | （PeerAuthentication なし） | STRICT 強制（peer-authentication.yaml） |

## 関連設計

- [ADR-MIG-002](../../../docs/02_構想設計/adr/ADR-MIG-002-istio-ambient-migration.md)
- [ADR-SEC-002](../../../docs/02_構想設計/adr/ADR-SEC-002-mtls-strict.md)
- [ADR-OBS-002](../../../docs/02_構想設計/adr/ADR-OBS-002-otel-pipeline.md) — Istio から OTel exporter へ
