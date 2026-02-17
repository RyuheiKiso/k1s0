# テンプレート仕様 — Kafka

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **Kafka メッセージング** リソースのテンプレート仕様を定義する。Strimzi KafkaTopic CRD（トピック定義）と Schema Registry Subject ConfigMap（スキーマ互換性設定）を、サービスの `tier` と `environment` に応じて自動生成する。

メッセージング設計の全体像は [メッセージング設計](メッセージング設計.md) を参照。

## 生成対象

| kind       | KafkaTopic | Schema Registry Subject |
| ---------- | ---------- | ----------------------- |
| `server`   | 条件付き   | 条件付き                |
| `bff`      | 条件付き   | 条件付き                |
| `client`   | 生成しない | 生成しない              |
| `library`  | 生成しない | 生成しない              |
| `database` | 生成しない | 生成しない              |

> server/bff で `has_kafka == true` の場合のみ生成する。

## 配置パス

生成されるリソースファイルは `infra/kafka/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル                | 配置パス                                                      |
| ----------------------- | ------------------------------------------------------------- |
| KafkaTopic              | `infra/kafka/{{ service_name }}/kafka-topic.yaml`             |
| Schema Registry Subject | `infra/kafka/{{ service_name }}/schema-registry-subject.yaml` |

## テンプレートファイル一覧

テンプレートは `CLI/templates/kafka/` 配下に配置する。

| テンプレートファイル                    | 生成先                                                        | 説明                              |
| --------------------------------------- | ------------------------------------------------------------- | --------------------------------- |
| `kafka-topic.yaml.tera`                 | `infra/kafka/{{ service_name }}/kafka-topic.yaml`             | Strimzi KafkaTopic CRD 定義      |
| `schema-registry-subject.yaml.tera`     | `infra/kafka/{{ service_name }}/schema-registry-subject.yaml` | Schema Registry Subject ConfigMap |

### ディレクトリ構成

```
CLI/
└── templates/
    └── kafka/
        ├── kafka-topic.yaml.tera
        └── schema-registry-subject.yaml.tera
```

## 使用するテンプレート変数

Kafka テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名         | 型      | KafkaTopic | Schema Registry Subject | 用途                                        |
| -------------- | ------- | ---------- | ----------------------- | ------------------------------------------- |
| `service_name` | String  | 用         | 用                      | リソース名、トピック名のドメイン部分        |
| `tier`         | String  | 用         | -                       | パーティション数の決定、トピック命名        |
| `namespace`    | String  | 用         | 用                      | リソースの配置先 Namespace                  |
| `domain`       | String  | 用         | 用                      | トピック命名のドメイン部分                  |
| `has_kafka`    | Boolean | 用         | 用                      | Kafka リソース生成の有無判定                |
| `environment`  | String  | 用         | -                       | レプリケーションファクターの決定            |

### Tier 別パーティション設定

| Tier       | デフォルトパーティション数 |
| ---------- | -------------------------- |
| `system`   | 6                          |
| `business` | 3                          |
| `service`  | 3                          |

### 環境別レプリケーション設定

| 環境      | レプリケーションファクター | min.insync.replicas |
| --------- | -------------------------- | ------------------- |
| `dev`     | 1                          | 1                   |
| `staging` | 1                          | 1                   |
| `prod`    | 3                          | 2                   |

---

## KafkaTopic テンプレート（kafka-topic.yaml.tera）

Strimzi KafkaTopic CRD を定義する。メイントピックと DLQ トピックを1ファイルに生成する。トピック命名規則、パーティション数、レプリケーション、保持ポリシーを `tier` と `environment` に応じて設定する。

```tera
apiVersion: kafka.strimzi.io/v1beta2
kind: KafkaTopic
metadata:
  name: k1s0.{{ tier }}.{{ domain }}.events.v1
  namespace: messaging
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
    strimzi.io/cluster: k1s0-kafka
spec:
{% if tier == "system" %}
  partitions: 6
{% else %}
  partitions: 3
{% endif %}
{% if environment == "prod" %}
  replicas: 3
  config:
    min.insync.replicas: "2"
{% else %}
  replicas: 1
  config:
    min.insync.replicas: "1"
{% endif %}
    retention.ms: "604800000"
    cleanup.policy: "delete"
---
# DLQ Topic
apiVersion: kafka.strimzi.io/v1beta2
kind: KafkaTopic
metadata:
  name: k1s0.{{ tier }}.{{ domain }}.events.v1.dlq
  namespace: messaging
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
    strimzi.io/cluster: k1s0-kafka
    k1s0/topic-type: dlq
spec:
{% if tier == "system" %}
  partitions: 6
{% else %}
  partitions: 3
{% endif %}
{% if environment == "prod" %}
  replicas: 3
  config:
    min.insync.replicas: "2"
{% else %}
  replicas: 1
  config:
    min.insync.replicas: "1"
{% endif %}
    retention.ms: "2592000000"
    cleanup.policy: "delete"
```

### ポイント

- **トピック命名**: `k1s0.{tier}.{domain}.events.v1` の規則に従う（[メッセージング設計](メッセージング設計.md) の D-119 準拠）
- **パーティション数**: system Tier は6パーティション、business/service Tier は3パーティションに設定する
- **レプリケーション**: prod 環境は3レプリカ・min.insync.replicas=2、dev/staging 環境は1レプリカ・min.insync.replicas=1
- **保持期間**: メイントピックは7日（604800000ms）、DLQ トピックは30日（2592000000ms）
- **DLQ トピック**: 元トピック名に `.dlq` サフィックスを付与する。処理失敗メッセージの保管用
- **Strimzi クラスター**: `strimzi.io/cluster: k1s0-kafka` ラベルで Kafka クラスターとの紐付けを宣言する

---

## Schema Registry Subject テンプレート（schema-registry-subject.yaml.tera）

Schema Registry の Subject 設定を ConfigMap として管理する。Confluent Schema Registry に登録する Subject の互換性モードを定義する。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name }}-schema-registry
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    app.kubernetes.io/component: schema-registry
data:
  subject-config.json: |
    {
      "subjects": [
        {
          "name": "k1s0.{{ tier }}.{{ domain }}.events.v1-value",
          "compatibilityLevel": "BACKWARD"
        }
      ],
      "schemaRegistryUrl": "http://schema-registry.k1s0-system.svc.cluster.local:8081"
    }
```

### ポイント

- **Subject 命名**: `{topic-name}-value` の規則に従う（[メッセージング設計](メッセージング設計.md) の Schema Registry セクション準拠）
- **互換性モード**: `BACKWARD` をデフォルトとし、コンシューマーの後方互換性を保証する
- **ConfigMap 方式**: Schema Registry の Subject 設定を Kubernetes ConfigMap で管理し、GitOps フローで適用する
- **エンドポイント**: Schema Registry は `k1s0-system` Namespace にデプロイされ、クラスタ内 DNS で参照する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                       | 選択肢                            | 生成への影響                                              |
| -------------------------- | --------------------------------- | --------------------------------------------------------- |
| kind (`kind`)              | `server` / `bff`                  | Kafka リソースの生成候補（`has_kafka` も必要）            |
| kind (`kind`)              | `client` / `library` / `database` | Kafka リソースを生成しない                                |
| Kafka 有効 (`has_kafka`)   | `true`                            | KafkaTopic + Schema Registry Subject を生成               |
| Kafka 有効 (`has_kafka`)   | `false`                           | Kafka リソースを生成しない                                |
| Tier (`tier`)              | `system`                          | パーティション数6                                        |
| Tier (`tier`)              | `business` / `service`            | パーティション数3                                        |
| 環境 (`environment`)       | `prod`                            | レプリカ3、min.insync.replicas=2                         |
| 環境 (`environment`)       | `dev` / `staging`                 | レプリカ1、min.insync.replicas=1                         |

---

## 生成例

### system Tier の Kafka 有効サーバー（prod 環境）の場合

入力:
```json
{
  "service_name": "auth-service",
  "tier": "system",
  "namespace": "k1s0-system",
  "domain": "auth",
  "has_kafka": true,
  "environment": "prod"
}
```

生成されるファイル:
- `infra/kafka/auth-service/kafka-topic.yaml` -- トピック `k1s0.system.auth.events.v1`、パーティション6、レプリカ3、min.insync.replicas=2
- `infra/kafka/auth-service/schema-registry-subject.yaml` -- Subject `k1s0.system.auth.events.v1-value`、互換性 BACKWARD

### service Tier の Kafka 有効サーバー（dev 環境）の場合

入力:
```json
{
  "service_name": "order-server",
  "tier": "service",
  "namespace": "k1s0-service",
  "domain": "order",
  "has_kafka": true,
  "environment": "dev"
}
```

生成されるファイル:
- `infra/kafka/order-server/kafka-topic.yaml` -- トピック `k1s0.service.order.events.v1`、パーティション3、レプリカ1、min.insync.replicas=1
- `infra/kafka/order-server/schema-registry-subject.yaml` -- Subject `k1s0.service.order.events.v1-value`、互換性 BACKWARD

### Kafka 無効のサーバーの場合

入力:
```json
{
  "service_name": "static-server",
  "tier": "service",
  "namespace": "k1s0-service",
  "domain": "static",
  "has_kafka": false,
  "environment": "dev"
}
```

生成されるファイル:
- なし（`has_kafka == false` のため Kafka リソースは生成されない）

---

## 関連ドキュメント

- [メッセージング設計](メッセージング設計.md) -- Kafka トピック設計・イベント駆動アーキテクチャの全体設計
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [config設計](config設計.md) -- config.yaml スキーマ（kafka セクション）
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) -- サーバーテンプレート（Kafka Producer/Consumer 連携）
- [テンプレート仕様-Helm](テンプレート仕様-Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-Config](テンプレート仕様-Config.md) -- Config テンプレート仕様
- [テンプレート仕様-Observability](テンプレート仕様-Observability.md) -- Observability テンプレート仕様
- [テンプレート仕様-CICD](テンプレート仕様-CICD.md) -- CI/CD テンプレート仕様
