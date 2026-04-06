# Vault Policy for config-server
# Tier: system
# Description: Provides access to config-server specific secrets and database credentials

# config-server static secrets
path "secret/data/k1s0/system/config-server/*" {
  capabilities = ["read"]
}

# config-server metadata
path "secret/metadata/k1s0/system/config-server/*" {
  capabilities = ["read", "list"]
}

# Shared database credentials (static)
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}

# Dynamic database credentials (read-write)
path "database/creds/config-server-rw" {
  capabilities = ["read"]
}

# Dynamic database credentials (read-only)
path "database/creds/config-server-ro" {
  capabilities = ["read"]
}

# PKI 証明書発行（サービス固有ロールに制限）
# 最小権限の原則: system tier 共通ロールではなく、config-server 専用ロールで発行する
path "pki_int/issue/config-server" {
  capabilities = ["create", "update"]
}

# Shared Kafka credentials
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}

# MEDIUM-INFRA-001 監査対応: システムリース更新権限を追加する
# 長時間稼働するサービスでリース期限切れによる接続断を防止する
path "sys/leases/renew" {
  capabilities = ["update"]
}

# 自身のトークン情報確認と更新のための権限
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}
