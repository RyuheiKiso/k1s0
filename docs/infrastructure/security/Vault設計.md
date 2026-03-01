# Vault 設計

D-006: Vault 戦略。認証方式、シークレットエンジン、パス体系、監査ログを定義する。

元ドキュメント: [認証認可設計.md](../../architecture/auth/認証認可設計.md)

---

## D-006: Vault 戦略

### バージョン

| 環境 | イメージ |
| ---- | -------- |
| ローカル開発 | `hashicorp/vault:1.17` |
| Kubernetes（本番） | Helm Chart 経由で Vault Operator をデプロイ（バージョンは `terraform設計.md` 参照） |

### 認証方式

| 認証方式         | 用途                                    |
| ---------------- | --------------------------------------- |
| Kubernetes Auth  | Kubernetes Pod からのアクセス           |
| AppRole          | CI/CD パイプラインからのアクセス        |
| LDAP             | 人間のオペレーターによる管理アクセス    |

### シークレットエンジン

| エンジン    | マウントパス       | 用途                                          |
| ----------- | ------------------ | --------------------------------------------- |
| KV v2       | `secret/`          | 静的シークレット（API キー、設定値等）        |
| Database    | `database/`        | データベースクレデンシャルの動的生成           |
| PKI         | `pki_int/`         | 内部 TLS 証明書の発行（Intermediate CA）      |

### シークレットパス体系

Vault に格納するシークレットのパス命名規則を定義する。全ドキュメントでこの規則に従うこと。

#### KV v2 パス命名規則

```
secret/data/k1s0/{tier}/{service}/{secret-type}
```

| 要素            | 説明                                | 例                              |
| --------------- | ----------------------------------- | ------------------------------- |
| `k1s0`          | プロジェクトプレフィックス（固定）  | ---                             |
| `tier`          | Tier 名                             | `system`, `business`, `service` |
| `service`       | サービス名（ハイフン区切り）        | `auth-server`, `order`, `bff-proxy` |
| `secret-type`   | シークレットの種別                  | `database`, `api-key`, `oidc`, `redis` |

#### Database シークレットエンジン命名規則

```
database/creds/{tier}-{service}-{permission}
```

| 要素            | 説明                          | 例                           |
| --------------- | ----------------------------- | ---------------------------- |
| `tier`          | Tier 名                       | `system`, `business`, `service` |
| `service`       | サービス名                    | `order`, `ledger`, `auth`    |
| `permission`    | 権限レベル                    | `rw`（読み書き）, `ro`（読み取り専用） |

#### PKI 証明書パス命名規則

```
pki_int/issue/{tier}
```

Root CA（`pki/`）は直接利用しない。Intermediate CA（`pki_int/`）経由で証明書を発行すること。

#### シークレットパス一覧

以下にプロジェクト内で使用する全シークレットとその Vault パスを定義する。

**system Tier**

- `secret/data/k1s0/system/auth-server/*` --- auth-server サービス固有シークレット（API キー、DB パスワード等）
- `secret/data/k1s0/system/config-server/*` --- config-server サービス固有シークレット
- `secret/data/k1s0/system/dlq-manager/*` --- dlq-manager サービス固有シークレット
- `secret/data/k1s0/system/saga-server/*` --- saga-server サービス固有シークレット
- `secret/data/k1s0/system/bff-proxy/*` --- bff-proxy サービス固有シークレット
- `secret/data/k1s0/system/redis/*` --- BFF セッション Redis AUTH パスワード（キー: `password`）
- `secret/data/k1s0/system/keycloak/bff-proxy` --- BFF OIDC Client Secret（キー: `client_secret`）
- `secret/data/k1s0/system/keycloak/*` --- Keycloak 統合シークレット（キー: `client_secret`）
- `secret/data/k1s0/system/database` --- 共有 DB 静的クレデンシャル（キー: `password`）
- `database/creds/auth-server-rw` --- auth-server DB 動的クレデンシャル（読み書き）
- `database/creds/auth-server-ro` --- auth-server DB 動的クレデンシャル（読み取り専用）
- `database/creds/config-server-rw` --- config-server DB 動的クレデンシャル（読み書き）
- `database/creds/config-server-ro` --- config-server DB 動的クレデンシャル（読み取り専用）

**business Tier**

- `secret/data/k1s0/business/{domain}/database` --- 各ドメイン DB パスワード（キー: `password`）
- `secret/data/k1s0/business/{domain}/api-key` --- 各ドメイン API キー（キー: `key`）
- `database/creds/business-{domain}-rw` --- 各ドメイン DB 動的クレデンシャル（読み書き）
- `database/creds/business-{domain}-ro` --- 各ドメイン DB 動的クレデンシャル（読み取り専用）

**service Tier**

- `secret/data/k1s0/service/{service}/database` --- 各サービス DB パスワード（キー: `password`）
- `secret/data/k1s0/service/{service}/api-key` --- 各サービス API キー（キー: `key`）
- `secret/data/k1s0/service/{service}/redis` --- 各サービス Redis AUTH パスワード（キー: `password`）
- `database/creds/service-{service}-rw` --- 各サービス DB 動的クレデンシャル（読み書き）
- `database/creds/service-{service}-ro` --- 各サービス DB 動的クレデンシャル（読み取り専用）

**共通（Kafka）**

- `secret/data/k1s0/system/kafka/sasl` --- Kafka SASL クレデンシャル（キー: `username`, `password`）

#### KV v2 シークレットのキー命名規則

各パスに格納するシークレットのキー名は以下の規則に従う。

| シークレット種別     | キー名            | 値の形式                |
| -------------------- | ----------------- | ----------------------- |
| DB パスワード        | `password`        | 文字列                  |
| API キー             | `key`             | 文字列                  |
| OIDC Client Secret   | `client_secret`   | 文字列                  |
| Redis AUTH パスワード | `password`       | 文字列                  |
| SASL ユーザー名      | `username`        | 文字列                  |
| SASL パスワード      | `password`        | 文字列                  |

#### Vault Agent Injector ファイルマウントパス規則

Pod にシークレットをファイルとして注入する際のマウントパスは以下の規則に従う。

```
/vault/secrets/{secret-type}
```

| マウントパス                    | 用途                          |
| ------------------------------- | ----------------------------- |
| `/vault/secrets/db-password`    | DB パスワード                 |
| `/vault/secrets/db-creds`       | DB 動的クレデンシャル         |
| `/vault/secrets/api-key`        | API キー                      |
| `/vault/secrets/redis-password` | Redis AUTH パスワード         |
| `/vault/secrets/oidc`           | OIDC Client Secret            |
| `/vault/secrets/kafka-sasl`     | Kafka SASL クレデンシャル     |

### Tier 別アクセスポリシー

ポリシーは **2階層構造** で管理する。Bootstrap 用の Tier ポリシー（Tier 全体へのワイルドカードアクセス）と、サービス固有のポリシー（最小権限）を組み合わせる。Kubernetes Auth ロールでは両者を `token_policies` に列挙する（例: `["k1s0-system", "auth-server"]`）。

#### Bootstrap 用 Tier ポリシー（`infra/vault/policies/k1s0-system.hcl`）

Tier 内の全シークレットへの読み取りアクセスを提供する。初期セットアップおよび Terraform Bootstrap 用途。

```hcl
# k1s0-system.hcl --- system tier 全体の Bootstrap ポリシー
path "secret/data/k1s0/system/*" {
  capabilities = ["read"]
}

path "secret/metadata/k1s0/system/*" {
  capabilities = ["read", "list"]
}
```

#### サービス固有ポリシー（`infra/vault/policies/{service}.hcl`）

各サービスが必要とするパスのみにアクセスを限定する最小権限ポリシー。以下は実装済みの4サービス分。

```hcl
# auth-server.hcl --- auth-server 固有ポリシー
path "secret/data/k1s0/system/auth-server/*" {
  capabilities = ["read"]
}
path "secret/metadata/k1s0/system/auth-server/*" {
  capabilities = ["read", "list"]
}
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}
path "database/creds/auth-server-rw" {
  capabilities = ["read"]
}
path "database/creds/auth-server-ro" {
  capabilities = ["read"]
}
path "pki_int/issue/system" {
  capabilities = ["create", "update"]
}
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}
path "secret/data/k1s0/system/keycloak/*" {
  capabilities = ["read"]
}

# config-server.hcl --- config-server 固有ポリシー
path "secret/data/k1s0/system/config-server/*" {
  capabilities = ["read"]
}
path "secret/metadata/k1s0/system/config-server/*" {
  capabilities = ["read", "list"]
}
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}
path "database/creds/config-server-rw" {
  capabilities = ["read"]
}
path "database/creds/config-server-ro" {
  capabilities = ["read"]
}
path "pki_int/issue/system" {
  capabilities = ["create", "update"]
}
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}

# dlq-manager.hcl --- dlq-manager 固有ポリシー
path "secret/data/k1s0/system/dlq-manager/*" {
  capabilities = ["read"]
}
path "secret/metadata/k1s0/system/dlq-manager/*" {
  capabilities = ["read", "list"]
}
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}

# saga-server.hcl --- saga-server 固有ポリシー
path "secret/data/k1s0/system/saga-server/*" {
  capabilities = ["read"]
}
path "secret/metadata/k1s0/system/saga-server/*" {
  capabilities = ["read", "list"]
}
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}

# bff-proxy.hcl --- bff-proxy 固有ポリシー
path "secret/data/k1s0/system/bff-proxy/*" {
  capabilities = ["read"]
}
path "secret/metadata/k1s0/system/bff-proxy/*" {
  capabilities = ["read", "list"]
}
path "secret/data/k1s0/system/redis/*" {
  capabilities = ["read"]
}
path "secret/data/k1s0/system/keycloak/bff-proxy" {
  capabilities = ["read"]
}
path "pki_int/issue/system" {
  capabilities = ["create", "update"]
}
path "secret/data/k1s0/system/service-auth/*" {
  capabilities = ["read"]
}
```

#### Terraform Bootstrap ポリシー（`infra/terraform/modules/vault/policies/`）

Terraform で管理する Tier レベルのポリシー。サービス固有ポリシー（`infra/vault/policies/`）と役割が異なることに注意。

```hcl
# policy/system.hcl --- system tier のポリシー（Terraform 管理）
path "secret/data/k1s0/system/*" {
  capabilities = ["read", "list"]
}
path "database/creds/system-*" {
  capabilities = ["read"]
}
path "pki_int/issue/system" {
  capabilities = ["create", "update"]
}

# policy/business.hcl --- business tier のポリシー
path "secret/data/k1s0/business/*" {
  capabilities = ["read", "list"]
}
path "database/creds/business-*" {
  capabilities = ["read"]
}
# Kafka SASL クレデンシャル（Tier 横断）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}

# policy/service.hcl --- service tier のポリシー
path "secret/data/k1s0/service/*" {
  capabilities = ["read", "list"]
}
path "database/creds/service-*" {
  capabilities = ["read"]
}
# Kafka SASL クレデンシャル（Tier 横断）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
```

### Kubernetes Auth 設定

Kubernetes Auth ロールはサービス固有の ServiceAccount を指定する（ワイルドカード SA は使用しない）。各ロールは Bootstrap 用 Tier ポリシーとサービス固有ポリシーの2つを付与する。

実装済みロール（`infra/vault/auth/` 配下）:

| ロール名        | ServiceAccount  | Namespace    | token_policies                    |
| --------------- | --------------- | ------------ | --------------------------------- |
| `auth-server`   | `auth-server`   | `k1s0-system` | `["k1s0-system", "auth-server"]`  |
| `config-server` | `config-server` | `k1s0-system` | `["k1s0-system", "config-server"]` |
| `dlq-manager`   | `dlq-manager`   | `k1s0-system` | `["k1s0-system", "dlq-manager"]`  |
| `saga-server`   | `saga-server`   | `k1s0-system` | `["k1s0-system", "saga-server"]`  |

> **注意**: `bff-proxy` の Kubernetes Auth YAML（`infra/vault/auth/k1s0-system-bff.yaml`）は未作成。bff-proxy を Vault 対応にする際は同様の ConfigMap を追加する必要がある。

各ロールの設定例（`infra/vault/auth/k1s0-system-auth.yaml` より）:

```json
{
  "role_name": "auth-server",
  "bound_service_account_names": ["auth-server"],
  "bound_service_account_namespaces": ["k1s0-system"],
  "token_policies": ["k1s0-system", "auth-server"],
  "token_ttl": "3600",
  "token_max_ttl": "86400"
}
```

### SecretProviderClass

Secrets Store CSI Driver 経由でシークレットを Pod にマウントする設定。`infra/vault/secret-provider-class/` に実装済み。

#### 実装済み SecretProviderClass 一覧

**auth-server**（`auth-secrets.yaml`）

| objectName           | secretPath                                        | secretKey  |
| -------------------- | ------------------------------------------------- | ---------- |
| `api-key`            | `secret/data/k1s0/system/auth-server/api-key`    | `key`      |
| `db-password`        | `secret/data/k1s0/system/auth-server/database`   | `password` |
| `kafka-sasl-username`| `secret/data/k1s0/system/kafka/sasl`             | `username` |
| `kafka-sasl-password`| `secret/data/k1s0/system/kafka/sasl`             | `password` |

**config-server**（`config-secrets.yaml`）

| objectName    | secretPath                                          | secretKey  |
| ------------- | --------------------------------------------------- | ---------- |
| `api-key`     | `secret/data/k1s0/system/config-server/api-key`    | `key`      |
| `db-password` | `secret/data/k1s0/system/config-server/database`   | `password` |

**dlq-manager**（`dlq-manager-secrets.yaml`）

| objectName           | secretPath                                         | secretKey  |
| -------------------- | -------------------------------------------------- | ---------- |
| `db-password`        | `secret/data/k1s0/system/dlq-manager/database`    | `password` |
| `kafka-sasl-username`| `secret/data/k1s0/system/kafka/sasl`              | `username` |
| `kafka-sasl-password`| `secret/data/k1s0/system/kafka/sasl`              | `password` |

**saga-server**（`saga-secrets.yaml`）

| objectName           | secretPath                                        | secretKey  |
| -------------------- | ------------------------------------------------- | ---------- |
| `db-password`        | `secret/data/k1s0/system/saga-server/database`   | `password` |
| `kafka-sasl-username`| `secret/data/k1s0/system/kafka/sasl`             | `username` |
| `kafka-sasl-password`| `secret/data/k1s0/system/kafka/sasl`             | `password` |

各 SPC の共通設定:
- `vaultAddress`: `http://vault.vault.svc.cluster.local:8200`
- `roleName`: サービス名（例: `auth-server`）
- `namespace`: `k1s0-system`

### Database シークレットエンジン

データベースクレデンシャルは Vault が動的に生成・ローテーションする。

```hcl
# modules/vault/database.tf

resource "vault_database_secret_backend" "db" {
  path = "database"
}

resource "vault_database_secret_backend_connection" "order_db" {
  backend       = vault_database_secret_backend.db.path
  name          = "service-order"
  allowed_roles = ["service-order-rw", "service-order-ro"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@postgres.k1s0-service.svc.cluster.local:5432/order_db"
  }
}

resource "vault_database_secret_backend_role" "order_rw" {
  backend             = vault_database_secret_backend.db.path
  name                = "service-order-rw"
  db_name             = vault_database_secret_backend_connection.order_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT ALL ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  default_ttl         = 86400     # 24 時間
  max_ttl             = 172800    # 48 時間
}
```

### 監査ログ設定

```hcl
# modules/vault/audit.tf

resource "vault_audit" "file" {
  type = "file"
  options = {
    file_path = "/vault/logs/audit.log"
    log_raw   = false              # シークレット値をマスク
  }
}
```

監査ログの内容:
- すべての認証試行（成功・失敗）
- すべてのシークレット読み取り操作
- ポリシー変更
- 設定変更

### クレデンシャルローテーション

| クレデンシャル種別     | ローテーション間隔 | 方式                        |
| ---------------------- | ------------------ | --------------------------- |
| DB パスワード          | 24 時間            | Vault Database エンジン自動 |
| API キー               | 90 日              | Vault KV v2 + 手動更新     |
| TLS 証明書             | 90 日              | Vault PKI エンジン自動     |
| JWT 署名鍵             | 90 日              | Keycloak の鍵ローテーション |

### Vault Agent Injector パターン

[helm設計.md](../kubernetes/helm設計.md) の Vault Agent Injector と連携して Pod にシークレットを注入する。

```yaml
# Deployment の annotations
spec:
  template:
    metadata:
      annotations:
        vault.hashicorp.com/agent-inject: "true"
        vault.hashicorp.com/role: "service"
        # 静的シークレット（KV v2）
        vault.hashicorp.com/agent-inject-secret-api-key: "secret/data/k1s0/service/order/api-key"
        # 動的シークレット（Database）
        vault.hashicorp.com/agent-inject-secret-db-creds: "database/creds/service-order-rw"
        vault.hashicorp.com/agent-inject-template-db-creds: |
          {{- with secret "database/creds/service-order-rw" -}}
          host=postgres.k1s0-service.svc.cluster.local
          port=5432
          dbname=order_db
          user={{ .Data.username }}
          password={{ .Data.password }}
          {{- end -}}
```

---

## 関連ドキュメント

- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- 基本方針・技術スタック
- [認証設計.md](../../architecture/auth/認証設計.md) -- OAuth 2.0 / OIDC 実装
- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC 設計
- [helm設計.md](../kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [terraform設計.md](../terraform/terraform設計.md) -- Terraform モジュール
