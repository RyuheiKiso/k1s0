# テンプレート仕様 — Config

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **環境別 config** のテンプレート仕様を定義する。server/bff の `config.yaml.tera` に加え、環境別オーバーライド（`config.dev.yaml.tera`、`config.staging.yaml.tera`、`config.prod.yaml.tera`）を定義し、開発・ステージング・本番環境に応じた設定を自動生成する。

config 設計の全体像は [config設計](../../cli/config/config設計.md) を参照。

## テンプレートファイル一覧

テンプレートは各 kind/lang ディレクトリの `config/` 配下に配置する。

### server/rust

| テンプレートファイル                            | 生成先                    | 説明                         |
| ----------------------------------------------- | ------------------------- | ---------------------------- |
| `server/rust/config/config.dev.yaml.tera`       | `config/config.dev.yaml`     | 開発環境用設定               |
| `server/rust/config/config.staging.yaml.tera`   | `config/config.staging.yaml` | ステージング環境用設定       |
| `server/rust/config/config.prod.yaml.tera`      | `config/config.prod.yaml`    | 本番環境用設定               |

### bff/go

| テンプレートファイル                        | 生成先                    | 説明                         |
| ------------------------------------------- | ------------------------- | ---------------------------- |
| `bff/go/config/config.dev.yaml.tera`       | `config/config.dev.yaml`     | 開発環境用設定               |
| `bff/go/config/config.staging.yaml.tera`   | `config/config.staging.yaml` | ステージング環境用設定       |
| `bff/go/config/config.prod.yaml.tera`      | `config/config.prod.yaml`    | 本番環境用設定               |

### bff/rust

| テンプレートファイル                          | 生成先                    | 説明                         |
| --------------------------------------------- | ------------------------- | ---------------------------- |
| `bff/rust/config/config.dev.yaml.tera`       | `config/config.dev.yaml`     | 開発環境用設定               |
| `bff/rust/config/config.staging.yaml.tera`   | `config/config.staging.yaml` | ステージング環境用設定       |
| `bff/rust/config/config.prod.yaml.tera`      | `config/config.prod.yaml`    | 本番環境用設定               |

### ディレクトリ構成

```
CLI/
└── templates/
    ├── server/
    │   ├── go/config/
    │   │   ├── config.dev.yaml.tera
    │   │   ├── config.staging.yaml.tera
    │   │   └── config.prod.yaml.tera
    │   └── rust/config/
    │       ├── config.dev.yaml.tera
    │       ├── config.staging.yaml.tera
    │       └── config.prod.yaml.tera
    └── bff/
        ├── go/config/
        │   ├── config.dev.yaml.tera
        │   ├── config.staging.yaml.tera
        │   └── config.prod.yaml.tera
        └── rust/config/
            ├── config.dev.yaml.tera
            ├── config.staging.yaml.tera
            └── config.prod.yaml.tera
```

## 使用するテンプレート変数

Config テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名               | 型       | dev | staging | prod | 用途                                     |
| -------------------- | -------- | --- | ------- | ---- | ---------------------------------------- |
| `service_name`       | String   | 用  | 用      | 用   | サービス名、ホスト名の生成               |
| `service_name_snake` | String   | 用  | 用      | 用   | 環境変数プレフィクス                     |
| `tier`               | String   | 用  | 用      | 用   | Namespace 導出、Tier 別デフォルト値      |
| `namespace`          | String   | 用  | 用      | 用   | リソースの配置先 Namespace               |
| `has_database`       | Boolean  | 用  | 用      | 用   | データベース接続設定の条件分岐           |
| `database_type`      | String   | 用  | 用      | 用   | データベース種別（postgres/mysql）       |
| `has_kafka`          | Boolean  | 用  | 用      | 用   | Kafka 接続設定の条件分岐                 |
| `has_redis`          | Boolean  | 用  | 用      | 用   | Redis 接続設定の条件分岐                 |
| `api_styles`         | [String] | 用  | 用      | 用   | API 方式に応じたポート・プロトコル設定   |

## 環境別差分

環境ごとに以下の設定値が異なる。

### 共通設定の環境別差分

| 設定項目              | dev                  | staging              | prod                        |
| --------------------- | -------------------- | -------------------- | --------------------------- |
| ログレベル            | `debug`              | `info`               | `warn`                      |
| トレース sample_rate  | `1.0`                | `0.1`                | `0.01`                      |
| SSL モード            | `disable`            | `require`            | `verify-full`               |
| シークレット管理      | 環境変数直接指定     | 環境変数直接指定     | Vault 参照                  |

### データベース接続の環境別差分

| 設定項目              | dev                  | staging              | prod                        |
| --------------------- | -------------------- | -------------------- | --------------------------- |
| ホスト                | `localhost`          | `{{ service_name_snake }}-db.staging` | Vault 参照         |
| ポート                | `5432` / `3306`      | `5432` / `3306`      | Vault 参照                  |
| SSL                   | `disable`            | `require`            | `verify-full`               |
| コネクションプール    | `max_connections: 5` | `max_connections: 20`| `max_connections: 50`       |

### Kafka 接続の環境別差分

| 設定項目              | dev                  | staging              | prod                        |
| --------------------- | -------------------- | -------------------- | --------------------------- |
| ブローカー            | `localhost:9092`     | `kafka.staging:9092` | Vault 参照                  |
| セキュリティ          | `PLAINTEXT`          | `SASL_SSL`           | `SASL_SSL`                  |

### Redis 接続の環境別差分

| 設定項目              | dev                  | staging              | prod                        |
| --------------------- | -------------------- | -------------------- | --------------------------- |
| ホスト                | `localhost`          | `redis.staging`      | Vault 参照                  |
| TLS                   | 無効                 | 有効                 | 有効                        |

---

## dev 環境テンプレート（config.dev.yaml.tera）

開発環境用の設定。デバッグに最適化された値を使用する。

```tera
server:
  name: {{ service_name }}
  port: {{ server_port | default(value=8080) }}
{% if api_styles is containing("grpc") %}
  grpc_port: {{ grpc_port | default(value=9090) }}
{% endif %}

logging:
  level: debug
  format: text

tracing:
  enabled: true
  sample_rate: 1.0
  exporter: stdout

{% if has_database %}
database:
  host: localhost
{% if database_type == "postgres" %}
  port: 5432
  driver: postgres
{% elif database_type == "mysql" %}
  port: 3306
  driver: mysql
{% endif %}
  name: {{ service_name_snake }}_dev
  user: dev_user
  password: dev_password
  ssl_mode: disable
  max_connections: 5
  idle_timeout: 30s
{% endif %}

{% if has_kafka %}
kafka:
  brokers:
    - localhost:9092
  security_protocol: PLAINTEXT
  consumer_group: {{ service_name_snake }}_dev
{% endif %}

{% if has_redis %}
redis:
  host: localhost
  port: 6379
  tls: false
  db: 0
{% endif %}
```

### ポイント

- ログレベルは `debug` に設定し、詳細なデバッグ情報を出力する
- トレースの sample_rate を `1.0`（全リクエスト）に設定し、開発時の問題追跡を容易にする
- データベース・Kafka・Redis は `localhost` に接続し、ローカル開発環境を想定する
- SSL は `disable` に設定し、ローカル開発の手軽さを優先する

---

## staging 環境テンプレート（config.staging.yaml.tera）

ステージング環境用の設定。本番相当の構成で動作検証を行う。

```tera
server:
  name: {{ service_name }}
  port: {{ server_port | default(value=8080) }}
{% if api_styles is containing("grpc") %}
  grpc_port: {{ grpc_port | default(value=9090) }}
{% endif %}

logging:
  level: info
  format: json

tracing:
  enabled: true
  sample_rate: 0.1
  exporter: otlp
  endpoint: otel-collector.{{ namespace }}:4317

{% if has_database %}
database:
  host: {{ service_name_snake }}-db.staging
{% if database_type == "postgres" %}
  port: 5432
  driver: postgres
{% elif database_type == "mysql" %}
  port: 3306
  driver: mysql
{% endif %}
  name: {{ service_name_snake }}
  user: {{ service_name_snake }}_user
  password: ${DB_PASSWORD}
  ssl_mode: require
  max_connections: 20
  idle_timeout: 60s
{% endif %}

{% if has_kafka %}
kafka:
  brokers:
    - kafka.staging:9092
  security_protocol: SASL_SSL
  consumer_group: {{ service_name_snake }}
{% endif %}

{% if has_redis %}
redis:
  host: redis.staging
  port: 6379
  tls: true
  db: 0
{% endif %}
```

### ポイント

- ログレベルは `info` に設定し、JSON 形式で構造化ログを出力する
- トレースの sample_rate を `0.1`（10%）に設定し、本番に近いサンプリング率で動作検証する
- SSL は `require` に設定し、本番相当のセキュリティを確保する
- パスワード等のシークレットは環境変数で注入する

---

## prod 環境テンプレート（config.prod.yaml.tera）

本番環境用の設定。セキュリティと安定性を最優先する。

```tera
server:
  name: {{ service_name }}
  port: {{ server_port | default(value=8080) }}
{% if api_styles is containing("grpc") %}
  grpc_port: {{ grpc_port | default(value=9090) }}
{% endif %}

logging:
  level: warn
  format: json

tracing:
  enabled: true
  sample_rate: 0.01
  exporter: otlp
  endpoint: otel-collector.{{ namespace }}:4317

{% if has_database %}
database:
  host: vault://secret/data/{{ namespace }}/{{ service_name_snake }}/db#host
{% if database_type == "postgres" %}
  port: 5432
  driver: postgres
{% elif database_type == "mysql" %}
  port: 3306
  driver: mysql
{% endif %}
  name: {{ service_name_snake }}
  user: vault://secret/data/{{ namespace }}/{{ service_name_snake }}/db#user
  password: vault://secret/data/{{ namespace }}/{{ service_name_snake }}/db#password
  ssl_mode: verify-full
  max_connections: 50
  idle_timeout: 120s
{% endif %}

{% if has_kafka %}
kafka:
  brokers:
    - vault://secret/data/{{ namespace }}/{{ service_name_snake }}/kafka#brokers
  security_protocol: SASL_SSL
  consumer_group: {{ service_name_snake }}
{% endif %}

{% if has_redis %}
redis:
  host: vault://secret/data/{{ namespace }}/{{ service_name_snake }}/redis#host
  port: 6379
  tls: true
  db: 0
{% endif %}
```

### ポイント

- ログレベルは `warn` に設定し、本番環境でのノイズを最小化する
- トレースの sample_rate を `0.01`（1%）に設定し、パフォーマンスへの影響を最小限に抑える
- SSL は `verify-full` に設定し、証明書の完全検証を行う
- シークレットは全て Vault 参照（`vault://secret/data/...`）を使用し、設定ファイルにシークレットを直接記載しない

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成される設定の内容が変わる。

| 条件                         | 選択肢                   | 生成への影響                                     |
| ---------------------------- | ------------------------ | ------------------------------------------------ |
| データベース (`has_database`) | `true`                   | database セクションを生成する                    |
| データベース種別 (`database_type`) | `postgres` / `mysql` | ドライバー・ポート番号の設定                     |
| Kafka (`has_kafka`)          | `true`                   | kafka セクションを生成する                       |
| Redis (`has_redis`)          | `true`                   | redis セクションを生成する                       |
| API 方式 (`api_styles`)      | `grpc` を含む            | gRPC ポート設定を追加する                        |

---

## 生成例

### dev 環境 — PostgreSQL + Kafka + Redis 使用の場合

入力:
```json
{
  "service_name": "order-service",
  "service_name_snake": "order_service",
  "tier": "service",
  "namespace": "k1s0-service",
  "has_database": true,
  "database_type": "postgres",
  "has_kafka": true,
  "has_redis": true,
  "api_styles": ["rest"]
}
```

生成されるファイル:
- `config/config.dev.yaml` -- localhost 接続、debug ログ、SSL disable
- `config/config.staging.yaml` -- staging ホスト接続、info ログ、SSL require
- `config/config.prod.yaml` -- Vault 参照、warn ログ、SSL verify-full

---

## 関連ドキュメント

- [config設計](../../cli/config/config設計.md) -- config の全体設計・スキーマ定義
- [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-サーバー](../server/サーバー.md) -- サーバーテンプレート仕様
- [テンプレート仕様-BFF](../client/BFF.md) -- BFF テンプレート仕様
- [テンプレート仕様-Dockerfile](../infrastructure/Dockerfile.md) -- Dockerfile テンプレート仕様
- [テンプレート仕様-Observability](../observability/Observability.md) -- Observability テンプレート仕様
- [テンプレート仕様-DockerCompose](../infrastructure/DockerCompose.md) -- Docker Compose テンプレート仕様
