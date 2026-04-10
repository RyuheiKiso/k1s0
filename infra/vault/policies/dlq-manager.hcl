# Vault Policy for dlq-manager
# Tier: system
# Description: Provides read access to dlq-manager specific secrets

# DLQ manager secrets
path "secret/data/k1s0/system/dlq-manager/*" {
  capabilities = ["read"]
}

# DLQ manager metadata
path "secret/metadata/k1s0/system/dlq-manager/*" {
  capabilities = ["read", "list"]
}

# Shared database credentials
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
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
