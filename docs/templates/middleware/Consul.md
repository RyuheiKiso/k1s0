# テンプレート仕様 — Consul

## 概要

k1s0 CLI ひな形生成のConsulテンプレート仕様。Terraform の State 管理に使用する Consul Backend の設定ファイルと、Consul Agent の基本設定を環境（`environment`）に応じて自動生成する。

Consul による State 管理の全体設計は [terraform設計](../../infrastructure/terraform/terraform設計.md) を、インフラ構成は [インフラ設計](../../infrastructure/overview/インフラ設計.md) を参照。

## 生成対象

| kind        | backend.tf | consul-config.yaml |
| ----------- | ---------- | ------------------ |
| `terraform` | 生成する   | 生成する           |

- Terraform 環境セットアップ時にのみ生成する。サーバー・クライアント等の通常のサービス生成では使用しない。
- 既存の Terraform テンプレート（`infra/terraform/environments/` 配下）への追加として扱う。

## 配置パス

生成されるリソースファイルは以下のパスに配置する。

| ファイル            | 配置パス                                              |
| ------------------- | ----------------------------------------------------- |
| backend.tf          | `infra/terraform/{{ environment }}/backend.tf`        |
| consul-config.yaml  | `infra/consul/consul-config.yaml`                     |

## テンプレートファイル一覧

テンプレートは `CLI/templates/consul/` 配下に配置する。

| テンプレートファイル          | 生成先                                                | 説明                              |
| ----------------------------- | ----------------------------------------------------- | --------------------------------- |
| `backend.tf.tera`             | `infra/terraform/{{ environment }}/backend.tf`        | Terraform Consul Backend 設定     |
| `consul-config.yaml.tera`     | `infra/consul/consul-config.yaml`                     | Consul Agent ConfigMap            |

### ディレクトリ構成

```
CLI/
└── templates/
    └── consul/
        ├── backend.tf.tera
        └── consul-config.yaml.tera
```

## 使用するテンプレート変数

Consul テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名         | 型     | backend.tf | consul-config.yaml | 用途                                       |
| -------------- | ------ | ---------- | ------------------- | ------------------------------------------ |
| `environment`  | String | 用         | 用                  | State パス、ログレベルの決定               |
| `tier`         | String | -          | 用                  | Consul サーバー構成の決定                  |
| `namespace`    | String | -          | 用                  | ConfigMap の配置先 Namespace               |

### 環境別 State パス

| environment | State パス                  |
| ----------- | --------------------------- |
| `dev`       | `terraform/k1s0/dev`        |
| `staging`   | `terraform/k1s0/staging`    |
| `prod`      | `terraform/k1s0/prod`       |

### 環境別 Consul 設定

| 設定           | dev     | staging | prod    |
| -------------- | ------- | ------- | ------- |
| log_level      | DEBUG   | WARN    | WARN    |
| server（モード）| false  | false   | false   |

---

## Backend 設定テンプレート（backend.tf.tera）

Terraform の Consul Backend 設定を定義する。環境ごとに異なる State パスを設定し、State の分離を保証する。

```tera
terraform {
  backend "consul" {
    address = "consul.internal.example.com:8500"
    scheme  = "https"
    path    = "terraform/k1s0/{{ environment }}"
    lock    = true
  }
}
```

### ポイント

- Consul のアドレスは `consul.internal.example.com:8500` を使用する（Kubernetes クラスタ外の共有 Consul サービス）
- `scheme` は `https` を使用し、通信を暗号化する
- `path` は `terraform/k1s0/{{ environment }}` の形式で環境ごとに State を分離する
- `lock = true` で State のロックを有効化し、同時実行による競合を防止する
- Consul は Ansible で構築・管理される独立したインフラであり、Terraform の管理対象外である

---

## Consul Agent ConfigMap テンプレート（consul-config.yaml.tera）

Kubernetes 上で Consul Agent を設定するための ConfigMap を定義する。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: consul-agent-config
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: consul
    app.kubernetes.io/component: agent
data:
  consul-config.json: |
    {
      "data_dir": "/opt/consul/data",
{% if environment == "dev" %}
      "log_level": "DEBUG",
{% else %}
      "log_level": "WARN",
{% endif %}
      "server": false,
      "retry_join": [
        "consul.internal.example.com"
      ],
      "bind_addr": "0.0.0.0",
      "client_addr": "0.0.0.0",
      "ports": {
        "http": 8500,
        "grpc": 8502
      },
      "telemetry": {
        "prometheus_retention_time": "30s"
      }
    }
```

### ポイント

- `data_dir` は `/opt/consul/data` に設定し、永続ボリュームにマウントする
- dev 環境では `log_level` を `DEBUG` に設定し、詳細なデバッグ情報を出力する
- staging/prod 環境では `log_level` を `WARN` に設定し、重要なイベントのみ記録する
- `server: false` で Agent モードとして動作させる（Server モードの Consul は Ansible で構築済み）
- `retry_join` でクラスタ外の Consul サーバーに自動接続する
- `telemetry.prometheus_retention_time` を設定し、Prometheus によるメトリクス収集を有効化する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                       | 選択肢                             | 生成への影響                                        |
| -------------------------- | ---------------------------------- | --------------------------------------------------- |
| 環境 (`environment`)       | `dev`                              | log_level=DEBUG                                     |
| 環境 (`environment`)       | `staging` / `prod`                 | log_level=WARN                                      |
| 環境 (`environment`)       | `dev` / `staging` / `prod`         | State パスが環境名で分離される                      |
| kind (`kind`)              | `terraform` 以外                   | Consul リソースを生成しない                         |

---

## 生成例

### dev 環境の場合

入力:
```json
{
  "environment": "dev",
  "tier": "system",
  "namespace": "consul"
}
```

生成されるファイル:
- `infra/terraform/dev/backend.tf` -- path=terraform/k1s0/dev、lock=true
- `infra/consul/consul-config.yaml` -- log_level=DEBUG

### staging 環境の場合

入力:
```json
{
  "environment": "staging",
  "tier": "system",
  "namespace": "consul"
}
```

生成されるファイル:
- `infra/terraform/staging/backend.tf` -- path=terraform/k1s0/staging、lock=true
- `infra/consul/consul-config.yaml` -- log_level=WARN

### prod 環境の場合

入力:
```json
{
  "environment": "prod",
  "tier": "system",
  "namespace": "consul"
}
```

生成されるファイル:
- `infra/terraform/prod/backend.tf` -- path=terraform/k1s0/prod、lock=true
- `infra/consul/consul-config.yaml` -- log_level=WARN

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [terraform設計](../../infrastructure/terraform/terraform設計.md) -- Terraform モジュール設計・State 管理・Consul Backend
- [インフラ設計](../../infrastructure/overview/インフラ設計.md) -- オンプレミスインフラ構成・共有サービス（Consul）
- [テンプレート仕様-Helm](../infrastructure/Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-CICD](../infrastructure/CICD.md) -- CI/CD テンプレート仕様
- [テンプレート仕様-Observability](../observability/Observability.md) -- Observability テンプレート仕様
- [可観測性設計](../../architecture/observability/可観測性設計.md) -- Prometheus メトリクス収集（Consul telemetry 連携）
