# infra/data/valkey — Valkey（Redis 互換 OSS KVS）

ADR-DATA-004 に従い、KVS を Valkey（Redis 互換 OSS）で運用する。
本ディレクトリは production-grade defaults を持ち、Argo CD で `valkey` namespace に展開される。

## ファイル

| ファイル | 内容 |
|---|---|
| `values.yaml` | Bitnami Valkey Helm chart の values（replication mode、Sentinel、PrometheusMetrics） |

## 利用するアプリ

- **State API**（KV store）— `t1-state` Pod が Dapr State Component 経由で参照
- **Dapr Workflow short-term backend** — `t1-workflow` Pod の短期 workflow 状態保存
- **Rate limiter / cache** — tier1 facade の per-tenant RPS 制御 / facade レスポンスキャッシュ

## デプロイ

```sh
helm repo add bitnami https://charts.bitnami.com/bitnami
helm upgrade --install valkey bitnami/valkey -n valkey --create-namespace -f infra/data/valkey/values.yaml --version 5.5.1
```

## ローカル開発との差分

| 観点 | dev（`tools/local-stack/manifests/75-valkey/`） | prod（本ディレクトリ） |
|---|---|---|
| architecture | standalone | replication（primary 1 + replica 2） |
| Sentinel | 無効 | 有効（自動フェイルオーバ） |
| auth | 無効 | 有効（OpenBao 動的パスワード） |
| persistence | 1Gi | 10Gi × 3（StorageClass 指定） |
| 監視 | 無効 | metrics.enabled + ServiceMonitor |

## 関連設計

- [ADR-DATA-004](../../../docs/02_構想設計/adr/ADR-DATA-004-valkey-kvs.md)
- FR-T1-STATE（KV store） / DS-SW-COMP-005（State Pod）
