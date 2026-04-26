# `tools/local-stack/` — kind ベースのローカル本番再現スタック

[IMP-DEV-POL-006](../../docs/05_実装/50_開発者体験設計/00_方針/01_開発者体験原則.md)（ローカルは kind/k3d + Dapr Local で本番再現）を実装するスタック。`tools/devcontainer/profiles/<role>/postCreateCommand` から起動され、tier1/tier2/tier3 の動作確認に必要な依存（Argo CD / cert-manager / SPIRE / Dapr operator / flagd / Backstage / Istio Ambient / Kyverno / CNPG / Kafka / MinIO / Valkey / OpenBao / 観測性）を kind 上に揃える。

## 配置

```
tools/local-stack/
├── README.md                     # 本ファイル
├── kind-cluster.yaml             # control-plane 1 + worker 3 + Calico CNI 対応
├── up.sh                         # 一括起動 (role 別 / layer 別 / skip 対応)
├── down.sh                       # 完全破棄
├── status.sh                     # 配備状態のサマリ
├── manifests/                    # 各レイヤの Helm values / kustomize
│   ├── 00-namespaces.yaml
│   ├── 20-cert-manager/
│   ├── 25-metallb/
│   ├── 30-istio-ambient/
│   ├── 35-kyverno/
│   ├── 40-spire/
│   ├── 45-dapr/
│   ├── 50-flagd/
│   ├── 55-argocd/
│   ├── 60-cnpg/
│   ├── 65-kafka/
│   ├── 70-minio/
│   ├── 75-valkey/
│   ├── 80-openbao/
│   ├── 85-backstage/
│   ├── 90-observability/
│   └── 95-keycloak/
├── dapr/components/              # Dapr Components (state / pub-sub / bindings / secret / config)
└── openbao-dev/                  # kind を使わない軽量 OpenBao スタンドアロン
```

## クイックスタート

```bash
# 役割を決めて kind + 本番再現スタックを起動
./tools/local-stack/up.sh --role tier1-rust-dev

# 状態確認
./tools/local-stack/status.sh

# 完全破棄（PV ごと消える）
./tools/local-stack/down.sh
```

役割別の既定配備セットは `up.sh` の `ROLE_LAYERS` で定義する。例えば `docs-writer` は CNI と cert-manager のみ、`infra-ops` / `full` は observability + Keycloak まで全載せ。

## 役割別の配備セット

| role | layers |
|---|---|
| tier1-rust-dev / tier1-go-dev / tier2-dev | core + dapr + cnpg + kafka + minio + valkey + openbao |
| tier3-web-dev | core + dapr + cnpg + openbao + backstage |
| tier3-native-dev | core + dapr + cnpg + openbao |
| platform-cli-dev | core + dapr + argocd + cnpg + backstage |
| sdk-dev | core + dapr + cnpg + kafka + openbao |
| infra-ops / full | 全レイヤ（observability + keycloak 含む） |
| docs-writer | cni + cert-manager のみ（軽量） |

`core` = cni + cert-manager + metallb + istio + kyverno + spire + dapr + flagd

## アクセスポイント

`up.sh` で起動後、kind の `extraPortMappings` 経由で host から直接アクセス可能:

| URL | 用途 | 認証 |
|---|---|---|
| http://localhost:30080 | Argo CD UI | admin / `kubectl -n argocd get secret argocd-initial-admin-secret -o json \| jq -r '.data.password' \| base64 -d` |
| http://localhost:30700 | Backstage | basic auth（dev image のみ） |
| http://localhost:30300 | Grafana | admin / k1s0-local-dev-password |

## Phase 3 残置項目

- `tools/local-stack/k3d-config.yaml`（個人開発日常用の軽量 k3d 経路）。kind と二択ではなく、起動時間と機能の trade-off を選ばせる前提だが、リリース時点では kind 単一でカバーできるため未配置とする。利用者から要望が顕在化した時点で配置する。
- 本 README 内のチャートバージョンは `up.sh` の `readonly *_VERSION` で固定している。Renovate 連動・GHCR digest 固定は Platform/Build 体制確立時の Phase 3 で導入する。
- LGTM 拡張（Mimir / Pyroscope）と SPIRE-Dapr 連携、Backstage への自前 image 差替えは Phase 3 で順次。

## 関連

- 設計: [`01_DevContainer_10役設計.md`](../../docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md)
- ホスト構成: [`01_WindowsWSL2環境構成.md`](../../docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md)
- ADR: [ADR-CICD-001](../../docs/02_構想設計/adr/ADR-CICD-001-argocd.md) / [ADR-FM-001](../../docs/02_構想設計/adr/ADR-FM-001-flagd-openfeature.md) / [ADR-SEC-002](../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md) / [ADR-SEC-003](../../docs/02_構想設計/adr/ADR-SEC-003-spiffe-spire.md) / [ADR-DATA-001](../../docs/02_構想設計/adr/ADR-DATA-001-cloudnativepg.md) / [ADR-DATA-002](../../docs/02_構想設計/adr/ADR-DATA-002-strimzi-kafka.md) / [ADR-DATA-003](../../docs/02_構想設計/adr/ADR-DATA-003-minio.md) / [ADR-DATA-004](../../docs/02_構想設計/adr/ADR-DATA-004-valkey.md) / [ADR-OBS-001](../../docs/02_構想設計/adr/ADR-OBS-001-grafana-lgtm.md) / [ADR-POL-001](../../docs/02_構想設計/adr/ADR-POL-001-kyverno-dual-ownership.md) / [ADR-BS-001](../../docs/02_構想設計/adr/ADR-BS-001-backstage.md)
- IMP-DEV-DC-014: ローカル Kubernetes と Dapr Local の統合
- IMP-DEV-DC-015: OpenBao dev server のローカル展開
