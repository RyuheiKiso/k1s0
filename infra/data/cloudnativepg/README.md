# infra/data/cloudnativepg — PostgreSQL（CloudNativePG operator）

ADR-DATA-001 に従い、PostgreSQL を CloudNativePG operator で運用する。
本ディレクトリは production-grade defaults を持ち、Argo CD で `cnpg-system` namespace に展開される。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | CloudNativePG operator Helm chart の values（HA、PodMonitor、リソース prod sizing） |
| `cluster.yaml` | アプリ用 PostgreSQL Cluster CRD（3 instance HA、PVC 100Gi、MinIO バックアップ、PodMonitor） |

## 利用するアプリ

- **State API**（relational store）— `t1-state` Pod が Dapr State Component 経由で参照
- **Audit ストア**（WORM）— `t1-audit` Pod が直接 SQL 接続、`audit_chain` table に append-only
- **Temporal バックエンド**（採用後の運用拡大時 移行）— `t1-workflow` Pod が長期 workflow に使用

## デプロイ

```sh
# operator install（cnpg-system namespace）
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm upgrade --install cnpg cnpg/cloudnative-pg -n cnpg-system --create-namespace -f infra/data/cloudnativepg/values.yaml

# アプリ用クラスタ作成
kubectl apply -f infra/data/cloudnativepg/cluster.yaml
```

production では Argo CD ApplicationSet 経由で適用する（plan 05-XX）。

## ローカル開発との差分

| 観点 | dev（`tools/local-stack/manifests/60-cnpg/`） | prod（本ディレクトリ） |
|---|---|---|
| operator replica | 1 | 3 |
| Cluster instances | 1 | 3（primary 1 + standby 2） |
| storage | 5Gi | 100Gi（StorageClass 指定） |
| monitoring | 無効 | PodMonitor + Grafana Dashboard |
| backup | なし | MinIO への定期スナップショット |
| 認証 | dev パスワード平文 | OpenBao 経由（plan 04-06） |

## 関連設計

- [ADR-DATA-001](../../../docs/02_構想設計/adr/ADR-DATA-001-postgres-cnpg.md) — PostgreSQL on CNPG 採用根拠
- DS-SW-COMP-007（t1-audit Pod、WORM ストア）
- FR-T1-STATE / FR-T1-AUDIT
