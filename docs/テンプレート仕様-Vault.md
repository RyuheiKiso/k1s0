# テンプレート仕様 — Vault

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **Vault シークレット管理** リソースのテンプレート仕様を定義する。SecretProviderClass（CSI Secrets Store Driver によるシークレット注入）、Vault Policy（サービス別最小権限ポリシー）、Vault Kubernetes Auth Config（認証ロール設定）を、サービスの `tier` と有効な機能フラグに応じて自動生成する。

シークレット管理の全体設計は [認証認可設計](認証認可設計.md) の「D-006: Vault 戦略」を参照。Vault パス体系は同ドキュメントの「シークレットパス体系」に準拠する。

## 生成対象

| kind       | SecretProviderClass | Vault Policy | Vault Auth |
| ---------- | ------------------- | ------------ | ---------- |
| `server`   | 生成する            | 生成する     | 生成する   |
| `bff`      | 生成する            | 生成する     | 生成する   |
| `client`   | 生成しない          | 生成しない   | 生成しない |
| `library`  | 生成しない          | 生成しない   | 生成しない |
| `database` | 生成しない          | 生成しない   | 生成しない |

> server/bff は常に生成する。すべてのサーバーコンポーネントにシークレット管理が必要なため。

## 配置パス

生成されるリソースファイルは `infra/vault/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル            | 配置パス                                                     |
| ------------------- | ------------------------------------------------------------ |
| SecretProviderClass | `infra/vault/{{ service_name }}/secret-provider-class.yaml`  |
| Vault Policy        | `infra/vault/{{ service_name }}/vault-policy.hcl`            |
| Vault Auth          | `infra/vault/{{ service_name }}/vault-auth.yaml`             |

## テンプレートファイル一覧

テンプレートは `CLI/templates/vault/` 配下に配置する。

| テンプレートファイル                    | 生成先                                                       | 説明                                     |
| --------------------------------------- | ------------------------------------------------------------ | ---------------------------------------- |
| `secret-provider-class.yaml.tera`       | `infra/vault/{{ service_name }}/secret-provider-class.yaml`  | SecretProviderClass（CSI Driver）        |
| `vault-policy.yaml.tera`               | `infra/vault/{{ service_name }}/vault-policy.hcl`            | Vault Policy（最小権限ポリシー）         |
| `vault-auth.yaml.tera`                 | `infra/vault/{{ service_name }}/vault-auth.yaml`             | Vault Kubernetes Auth Config             |

### ディレクトリ構成

```
CLI/
└── templates/
    └── vault/
        ├── secret-provider-class.yaml.tera
        ├── vault-policy.yaml.tera
        └── vault-auth.yaml.tera
```

## 使用するテンプレート変数

Vault テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名          | 型      | SecretProviderClass | Vault Policy | Vault Auth | 用途                                        |
| --------------- | ------- | ------------------- | ------------ | ---------- | ------------------------------------------- |
| `service_name`  | String  | 用                  | 用           | 用         | リソース名、ポリシー名、サービスアカウント名 |
| `tier`          | String  | 用                  | 用           | -          | シークレットパスの Tier 部分                |
| `namespace`     | String  | 用                  | -            | 用         | リソースの配置先 Namespace                  |
| `has_database`  | Boolean | 用                  | 用           | -          | DB シークレットパスの生成有無               |
| `has_kafka`     | Boolean | 用                  | 用           | -          | Kafka SASL シークレットパスの生成有無       |
| `has_redis`     | Boolean | 用                  | 用           | -          | Redis シークレットパスの生成有無            |
| `database_type` | String  | 用                  | -            | -          | DB シークレットパスの決定（postgresql 等）  |

### シークレットパス体系（認証認可設計.md 準拠）

| シークレット種別   | Vault パス                                           | 条件             |
| ------------------ | ---------------------------------------------------- | ---------------- |
| DB パスワード      | `secret/data/k1s0/{tier}/{service_name}/database`    | `has_database`   |
| Redis パスワード   | `secret/data/k1s0/{tier}/{service_name}/redis`       | `has_redis`      |
| Kafka SASL         | `secret/data/k1s0/system/kafka/sasl`                 | `has_kafka`      |
| API キー           | `secret/data/k1s0/{tier}/{service_name}/api-key`     | 常に生成         |

---

## SecretProviderClass テンプレート（secret-provider-class.yaml.tera）

CSI Secrets Store Driver を使用して Vault からシークレットを Pod に注入する SecretProviderClass リソースを定義する。サービスが利用する機能フラグ（`has_database`、`has_redis`、`has_kafka`）に応じて、注入するシークレットのパスを動的に構成する。

```tera
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: {{ service_name }}-vault-secrets
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
spec:
  provider: vault
  parameters:
    vaultAddress: "http://vault.vault.svc.cluster.local:8200"
    roleName: "{{ service_name }}"
    objects: |
      - objectName: "api-key"
        secretPath: "secret/data/k1s0/{{ tier }}/{{ service_name }}/api-key"
        secretKey: "key"
{% if has_database %}
      - objectName: "db-password"
        secretPath: "secret/data/k1s0/{{ tier }}/{{ service_name }}/database"
        secretKey: "password"
{% endif %}
{% if has_redis %}
      - objectName: "redis-password"
        secretPath: "secret/data/k1s0/{{ tier }}/{{ service_name }}/redis"
        secretKey: "password"
{% endif %}
{% if has_kafka %}
      - objectName: "kafka-sasl-username"
        secretPath: "secret/data/k1s0/system/kafka/sasl"
        secretKey: "username"
      - objectName: "kafka-sasl-password"
        secretPath: "secret/data/k1s0/system/kafka/sasl"
        secretKey: "password"
{% endif %}
```

### ポイント

- **CSI Secrets Store Driver**: Vault Agent Injector の代替として、CSI ボリュームドライバ経由でシークレットを Pod にマウントする
- **API キー**: すべてのサーバーに対して常に API キーのシークレットパスを生成する
- **DB シークレット**: `has_database == true` の場合のみ DB パスワードのパスを追加する
- **Redis シークレット**: `has_redis == true` の場合のみ Redis パスワードのパスを追加する
- **Kafka SASL**: `has_kafka == true` の場合のみ Kafka SASL クレデンシャル（username + password）のパスを追加する。Kafka SASL は共通基盤のため `secret/data/k1s0/system/kafka/sasl` を参照する
- **Vault ロール名**: サービス名をそのまま Vault ロール名として使用する

---

## Vault Policy テンプレート（vault-policy.yaml.tera）

サービスごとに最小権限の Vault ポリシーを生成する。サービスが必要とするシークレットパスのみに `read` 権限を付与する。

```tera
# Vault Policy for {{ service_name }}
# Tier: {{ tier }}

# API key (always required)
path "secret/data/k1s0/{{ tier }}/{{ service_name }}/api-key" {
  capabilities = ["read"]
}

{% if has_database %}
# Database credentials (static)
path "secret/data/k1s0/{{ tier }}/{{ service_name }}/database" {
  capabilities = ["read"]
}

# Database credentials (dynamic)
path "database/creds/{{ tier }}-{{ service_name }}-rw" {
  capabilities = ["read"]
}

path "database/creds/{{ tier }}-{{ service_name }}-ro" {
  capabilities = ["read"]
}
{% endif %}

{% if has_redis %}
# Redis credentials
path "secret/data/k1s0/{{ tier }}/{{ service_name }}/redis" {
  capabilities = ["read"]
}
{% endif %}

{% if has_kafka %}
# Kafka SASL credentials (shared across tiers)
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
{% endif %}
```

### ポイント

- **最小権限の原則**: サービスが実際に必要とするシークレットパスにのみ `read` 権限を付与する
- **capabilities は read のみ**: アプリケーションサービスはシークレットの読み取りのみ許可し、書き込み・削除は禁止する
- **動的クレデンシャル**: `has_database == true` の場合、静的パスワード（KV v2）と動的クレデンシャル（Database シークレットエンジン）の両方のパスに権限を付与する
- **Kafka SASL の Tier 横断**: Kafka SASL クレデンシャルは system Tier の共通パスを参照する。business/service Tier のサービスも `secret/data/k1s0/system/kafka/sasl` を読み取れるようにする（[認証認可設計](認証認可設計.md) の Tier 別アクセスポリシー準拠）

---

## Vault Auth テンプレート（vault-auth.yaml.tera）

Vault の Kubernetes Auth Method のロール設定を定義する。サービスアカウントと Namespace の紐付けにより、Pod が Vault に認証できるようにする。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name }}-vault-auth
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    app.kubernetes.io/component: vault-auth
data:
  vault-auth-config.json: |
    {
      "role_name": "{{ service_name }}",
      "bound_service_account_names": ["{{ service_name }}"],
      "bound_service_account_namespaces": ["{{ namespace }}"],
      "token_policies": ["{{ service_name }}"],
      "token_ttl": "3600",
      "token_max_ttl": "86400"
    }
```

### ポイント

- **Kubernetes Auth Method**: Pod の ServiceAccount トークンを使用して Vault に認証する
- **bound_service_account_names**: サービス名と同名の ServiceAccount のみ認証を許可する
- **bound_service_account_namespaces**: サービスが所属する Namespace からの認証のみ許可する
- **token_policies**: サービス名と同名の Vault ポリシーを適用する（vault-policy.yaml.tera で生成したポリシー）
- **token_ttl**: 1時間（3600秒）。Vault トークンの有効期限
- **token_max_ttl**: 24時間（86400秒）。トークン更新時の最大有効期限
- **ConfigMap 方式**: Vault Auth 設定を ConfigMap で管理し、Terraform や Vault CLI で適用する際の入力パラメータとして使用する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                          | 選択肢                            | 生成への影響                                              |
| ----------------------------- | --------------------------------- | --------------------------------------------------------- |
| kind (`kind`)                 | `server` / `bff`                  | Vault リソースを生成する                                  |
| kind (`kind`)                 | `client` / `library` / `database` | Vault リソースを生成しない                                |
| DB 有効 (`has_database`)      | `true`                            | SecretProviderClass に DB パス追加、Policy に DB 権限追加 |
| DB 有効 (`has_database`)      | `false`                           | DB 関連のパス・権限を省略                                |
| Kafka 有効 (`has_kafka`)      | `true`                            | SecretProviderClass に Kafka SASL パス追加、Policy に Kafka 権限追加 |
| Kafka 有効 (`has_kafka`)      | `false`                           | Kafka 関連のパス・権限を省略                              |
| Redis 有効 (`has_redis`)      | `true`                            | SecretProviderClass に Redis パス追加、Policy に Redis 権限追加 |
| Redis 有効 (`has_redis`)      | `false`                           | Redis 関連のパス・権限を省略                              |

---

## 生成例

### system Tier の全機能有効サーバーの場合

入力:
```json
{
  "service_name": "auth-service",
  "tier": "system",
  "namespace": "k1s0-system",
  "has_database": true,
  "has_kafka": true,
  "has_redis": true,
  "database_type": "postgresql"
}
```

生成されるファイル:
- `infra/vault/auth-service/secret-provider-class.yaml` -- API キー + DB パスワード + Redis パスワード + Kafka SASL（4種のシークレット）
- `infra/vault/auth-service/vault-policy.hcl` -- API キー + DB（静的・動的） + Redis + Kafka SASL の read 権限
- `infra/vault/auth-service/vault-auth.yaml` -- ServiceAccount `auth-service`、Namespace `k1s0-system`

### service Tier の DB のみ有効なサーバーの場合

入力:
```json
{
  "service_name": "order-server",
  "tier": "service",
  "namespace": "k1s0-service",
  "has_database": true,
  "has_kafka": false,
  "has_redis": false,
  "database_type": "postgresql"
}
```

生成されるファイル:
- `infra/vault/order-server/secret-provider-class.yaml` -- API キー + DB パスワード（2種のシークレット）
- `infra/vault/order-server/vault-policy.hcl` -- API キー + DB（静的・動的）の read 権限のみ
- `infra/vault/order-server/vault-auth.yaml` -- ServiceAccount `order-server`、Namespace `k1s0-service`

### business Tier の最小構成サーバーの場合

入力:
```json
{
  "service_name": "ledger-server",
  "tier": "business",
  "namespace": "k1s0-business",
  "has_database": false,
  "has_kafka": false,
  "has_redis": false,
  "database_type": ""
}
```

生成されるファイル:
- `infra/vault/ledger-server/secret-provider-class.yaml` -- API キーのみ（1種のシークレット）
- `infra/vault/ledger-server/vault-policy.hcl` -- API キーの read 権限のみ
- `infra/vault/ledger-server/vault-auth.yaml` -- ServiceAccount `ledger-server`、Namespace `k1s0-business`

---

## 関連ドキュメント

- [認証認可設計](認証認可設計.md) -- Vault 戦略・シークレットパス体系・Kubernetes Auth 設定
- [config設計](config設計.md) -- config.yaml スキーマ（シークレット管理セクション）
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [helm設計](helm設計.md) -- Helm Chart・Vault Agent Injector 連携
- [terraform設計](terraform設計.md) -- Terraform モジュール（vault/ モジュール）
- [テンプレート仕様-Observability](テンプレート仕様-Observability.md) -- Observability テンプレート仕様
- [テンプレート仕様-Helm](テンプレート仕様-Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-Config](テンプレート仕様-Config.md) -- Config テンプレート仕様
- [テンプレート仕様-CICD](テンプレート仕様-CICD.md) -- CI/CD テンプレート仕様
