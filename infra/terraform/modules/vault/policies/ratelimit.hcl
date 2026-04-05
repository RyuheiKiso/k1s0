# ratelimit サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、ratelimit サービスが必要なパスのみに限定する。
# ratelimit サービスはレート制限の設定・カウンタ管理を担う。
# Redis（カウンタストア）と DB（設定値）クレデンシャルが必要。

# ratelimit サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/ratelimit/*" {
  capabilities = ["read"]
}

# ratelimit シークレットのメタデータ参照
path "secret/metadata/k1s0/system/ratelimit/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（ratelimit DB のみ）
path "database/creds/system-ratelimit" {
  capabilities = ["read"]
}

# Redis クレデンシャル（レートカウンタストア）
path "secret/data/k1s0/system/redis" {
  capabilities = ["read"]
}
