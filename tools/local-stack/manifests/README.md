# `tools/local-stack/manifests/` — 本番再現スタックの manifest 群

`tools/local-stack/up.sh` から段階的に配備される本番再現コンポーネントの Helm values と Kustomize manifest を保持する。配備順は接頭辞数値で固定し、依存関係（CNI → CRD → Operator → Workload）の順を守る。

## レイヤ一覧

| 接頭辞 | レイヤ | 内容 | 主 ADR |
|---|---|---|---|
| `00-namespaces.yaml` | 前提 | 全 namespace の事前作成 | — |
| `10-cni-calico/` | CNI | Calico (kindnet 不採用) | — |
| `20-cert-manager/` | TLS | cert-manager + ClusterIssuer (selfsigned/CA) | — |
| `25-metallb/` | LB | MetalLB IPAddressPool / L2Advertisement | ADR-STOR-002 |
| `30-istio-ambient/` | mesh | Istio Ambient (CNI + ztunnel) | ADR-0003 |
| `35-kyverno/` | policy | Kyverno admission + background controller | ADR-POL-001 |
| `40-spire/` | identity | SPIRE server + agent | ADR-SEC-003 |
| `45-dapr/` | dapr | Dapr operator + control plane | ADR-TIER1-001 |
| `50-flagd/` | feature flag | flagd デーモン (ConfigMap ベース) | ADR-FM-001 |
| `55-argocd/` | gitops | Argo CD (NodePort 30080) | ADR-CICD-001 |
| `60-cnpg/` | data | CloudNativePG operator + k1s0 cluster | ADR-DATA-001 |
| `65-kafka/` | data | Strimzi operator + KRaft 単一ノード | ADR-DATA-002 |
| `70-minio/` | data | MinIO standalone | ADR-DATA-003 |
| `75-valkey/` | data | Valkey standalone | ADR-DATA-004 |
| `80-openbao/` | secret | OpenBao dev mode | ADR-SEC-002 |
| `85-backstage/` | dev portal | Backstage (NodePort 30700) | ADR-BS-001 |
| `90-observability/` | obs | Grafana + Loki + Tempo (NodePort 30300) | ADR-OBS-001 |
| `95-keycloak/` | identity | Keycloak (CNPG 連携) | ADR-SEC-001 |

## 利用

`up.sh` から呼ばれることを前提とする。手動で個別レイヤを当てる場合は接頭辞順を守る:

```bash
kubectl apply -f tools/local-stack/manifests/00-namespaces.yaml
helm install cert-manager jetstack/cert-manager \
    -n cert-manager --version v1.16.2 \
    -f tools/local-stack/manifests/20-cert-manager/values.yaml
# ... 以下同様
```

## 設計ノート

- ローカル dev は HA を切る（replica=1 / persistence は最小 1-5Gi）。本番は ArgoCD で別 manifest を当てる。
- 認証情報はリポジトリにベタ書き。これは **ローカル kind 専用** であり、本番は ExternalSecrets / OpenBao 経由で注入する。
- リソース要求は 48GB RAM ホストでの「スタック全部入り + Dev Container + Rust build」を想定し、各 Pod を 50-200MB に抑える。
