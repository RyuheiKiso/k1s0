# Vault 設計

D-006: Vault 戦略。認証方式、シークレットエンジン、パス体系、監査ログを定義する。

元ドキュメント: [認証認可設計.md](../../architecture/auth/認証認可設計.md)

---

## D-006: Vault 戦略

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
| PKI         | `pki/`             | 内部 TLS 証明書の発行                         |

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
| `service`       | サービス名（ハイフン区切り）        | `auth`, `order`, `bff`          |
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
pki/issue/{tier}
```

#### シークレットパス一覧

以下にプロジェクト内で使用する全シークレットとその Vault パスを定義する。

**system Tier**

- `secret/data/k1s0/system/auth/oidc` --- Keycloak OIDC Client Secret（キー: `client_secret`）
- `secret/data/k1s0/system/auth/database` --- 認証サービス DB パスワード（キー: `password`）
- `secret/data/k1s0/system/bff/redis` --- BFF セッション Redis AUTH パスワード（キー: `password`）
- `secret/data/k1s0/system/bff/oidc` --- BFF OIDC Client Secret（キー: `client_secret`）
- `database/creds/system-auth-rw` --- 認証サービス DB 動的クレデンシャル（読み書き）
- `database/creds/system-auth-ro` --- 認証サービス DB 動的クレデンシャル（読み取り専用）

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

各 Tier のサービスがアクセスできる Vault パスを制限する。

```hcl
# policy/system.hcl --- system tier のポリシー
path "secret/data/k1s0/system/*" {
  capabilities = ["read", "list"]
}
path "database/creds/system-*" {
  capabilities = ["read"]
}
path "pki/issue/system" {
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

```hcl
# Kubernetes Auth Backend の設定
resource "vault_auth_backend" "kubernetes" {
  type = "kubernetes"
}

resource "vault_kubernetes_auth_backend_config" "k8s" {
  backend            = vault_auth_backend.kubernetes.path
  kubernetes_host    = "https://kubernetes.default.svc"
  kubernetes_ca_cert = file("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")
}

# system tier のロール
resource "vault_kubernetes_auth_backend_role" "system" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "system"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-system"]
  token_policies                   = ["system"]
  token_ttl                        = 3600
}

# business tier のロール
resource "vault_kubernetes_auth_backend_role" "business" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "business"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-business"]
  token_policies                   = ["business"]
  token_ttl                        = 3600
}

# service tier のロール
resource "vault_kubernetes_auth_backend_role" "service" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "service"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-service"]
  token_policies                   = ["service"]
  token_ttl                        = 3600
}
```

上記の Terraform 設定は [terraform設計.md](../terraform/terraform設計.md) の `modules/vault/` に配置する。

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
