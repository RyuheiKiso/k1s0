# Vault Policy for bff-proxy
# Tier: system
# Description: Provides access to bff-proxy specific secrets and PKI certificates

# bff-proxy static secrets
path "secret/data/k1s0/system/bff-proxy/*" {
  capabilities = ["read"]
}

# bff-proxy metadata
path "secret/metadata/k1s0/system/bff-proxy/*" {
  capabilities = ["read", "list"]
}

# Session store credentials (Redis)
path "secret/data/k1s0/system/redis/*" {
  capabilities = ["read"]
}

# Keycloak OIDC client secrets
path "secret/data/k1s0/system/keycloak/bff-proxy" {
  capabilities = ["read"]
}

# PKI 証明書発行（サービス固有ロールに制限）
# 最小権限の原則: system tier 共通ロールではなく、bff-proxy 専用ロールで発行する
path "pki_int/issue/bff-proxy" {
  capabilities = ["create", "update"]
}

# Upstream service auth secrets
path "secret/data/k1s0/system/service-auth/*" {
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
