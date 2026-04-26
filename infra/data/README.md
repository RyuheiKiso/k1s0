# infra/data — 永続データ層 4 セット

k1s0 のデータ永続化層 4 種を Kubernetes operator ベースで配置する。本ディレクトリの
manifests は **production-grade defaults** を持ち、環境別差分（dev / staging / prod）は
`infra/environments/<env>/` の overlay で上書きする運用とする。

## 含まれる backend

| ディレクトリ | 用途 | Operator / Chart | ADR |
|---|---|---|---|
| [`cloudnativepg/`](cloudnativepg/) | リレーショナル DB（PostgreSQL）。State API の relational store / Audit WORM ストア / Workflow（Temporal）バックエンドに使う。 | [CloudNativePG](https://cloudnative-pg.io/) | [ADR-DATA-001](../../docs/02_構想設計/adr/ADR-DATA-001-postgres-cnpg.md) |
| [`kafka/`](kafka/) | イベントストリーム / PubSub backend。 | [Strimzi Kafka Operator](https://strimzi.io/) | [ADR-DATA-002](../../docs/02_構想設計/adr/ADR-DATA-002-kafka-strimzi.md) |
| [`minio/`](minio/) | S3 互換オブジェクトストレージ。Binding / Backup / Artifacts に使う。 | [MinIO Helm chart](https://github.com/minio/minio/tree/master/helm/minio) | [ADR-DATA-003](../../docs/02_構想設計/adr/ADR-DATA-003-minio-s3.md) |
| [`valkey/`](valkey/) | KVS（Redis 互換 OSS）。State API の KV store / Dapr Workflow short-term backend に使う。 | [Bitnami Valkey](https://github.com/bitnami/charts/tree/main/bitnami/valkey) | [ADR-DATA-004](../../docs/02_構想設計/adr/ADR-DATA-004-valkey-kvs.md) |

## ローカル開発との関係

`tools/local-stack/manifests/{60-cnpg,65-kafka,70-minio,75-valkey}/` には kind 単一ノードで
最小構成（HA なし / 1 replica / 5Gi）の dev values が配置されている。本 `infra/data/` は
それを **production-grade defaults** に正規化したもの（HA / 3+ replica / 監視 enabled /
バックアップ enabled / リソース sizing 拡大）。

| 観点 | local-stack（dev） | infra/data（prod default） |
|---|---|---|
| operator replica | 1 | 3（HA） |
| データ instance | 1（standalone） | 3+（cluster / replication） |
| ストレージ | 5Gi 程度 | 100Gi 以上、StorageClass 指定 |
| 監視 | 無効 | 有効（podMonitor / PrometheusRule） |
| バックアップ | なし | MinIO / S3 への定期スナップショット |
| 認証 | dev パスワード平文 | OpenBao 経由の動的シークレット（plan 04-06） |

## 配備フロー（Argo CD）

`deploy/apps/application-sets/` 配下の ApplicationSet が本ディレクトリを ref する形で
Argo CD に Application を作る。環境別に `infra/environments/<env>/data-overlay/`（plan 05-XX 予定）
の Kustomize overlay で values を上書きする。

```
deploy/apps/application-sets/data-cnpg.yaml   --refs--> infra/data/cloudnativepg/
                                                            └ values.yaml + cluster.yaml
                                              --overlay--> infra/environments/<env>/data-cnpg/
```

## 未着手項目（plan 05-XX 以降）

- 各 backend の `helmrelease.yaml`（Argo CD Application の中身、本リリース時点 は values.yaml と manifest のみ）
- `infra/environments/{dev,staging,prod}/data-*` の overlay 雛形
- Backup / Restore の Runbook（`docs/40_運用ライフサイクル/`）
- 認証は OpenBao の動的 secret に切替（plan 04-06）
