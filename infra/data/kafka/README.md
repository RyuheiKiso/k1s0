# infra/data/kafka — Kafka（Strimzi operator）

ADR-DATA-002 に従い、Kafka を Strimzi operator で運用する。
本ディレクトリは production-grade defaults を持ち、Argo CD で `kafka` namespace に展開される。

## ファイル

| ファイル | 内容 |
|---|---|
| `strimzi-values.yaml` | Strimzi operator Helm chart の values（PrometheusRule、リソース prod sizing） |
| `kafka-cluster.yaml` | アプリ用 Kafka Cluster + KafkaNodePool（KRaft、3 broker HA、PVC 100Gi、TLS listener） |

## 利用するアプリ

- **PubSub API** — `t1-state` Pod が Dapr PubSub Component 経由で参照
- **Audit イベント連携**（リアルタイム監査配信）— `t1-audit` Pod から topic publish
- **採用後の運用拡大時** はマルチテナント分離 / partition 増設

## デプロイ

```sh
# operator install（kafka namespace、watchAnyNamespace=true で全 namespace 監視）
helm repo add strimzi https://strimzi.io/charts/
helm upgrade --install strimzi strimzi/strimzi-kafka-operator -n kafka --create-namespace -f infra/data/kafka/strimzi-values.yaml --version 0.51.0

# Kafka Cluster + NodePool 作成
kubectl apply -f infra/data/kafka/kafka-cluster.yaml
```

## ローカル開発との差分

| 観点 | dev（`tools/local-stack/manifests/65-kafka/`） | prod（本ディレクトリ） |
|---|---|---|
| operator replica | 1 | 1（PrometheusRule 有効化） |
| KafkaNodePool replicas | 1（dual-role） | 3（controller + broker 分離も可） |
| storage | 5Gi | 100Gi（StorageClass 指定） |
| listeners | plain（TLS なし） | tls（mTLS 必須、cert-manager 統合） |
| replication.factor | 1 | 3 |
| min.insync.replicas | 1 | 2 |

## 関連設計

- [ADR-DATA-002](../../../docs/02_構想設計/adr/ADR-DATA-002-kafka-strimzi.md)
- FR-T1-PUBSUB
