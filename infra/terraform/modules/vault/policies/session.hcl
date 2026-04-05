# session サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、session サービスが必要なパスのみに限定する。
# session サービスはセッショントークンの管理を担う。
# Transit による暗号化・復号化、および session 専用 DB クレデンシャルが必要。

# session サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/session/*" {
  capabilities = ["read"]
}

# session シークレットのメタデータ参照
path "secret/metadata/k1s0/system/session/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（session DB のみ）
path "database/creds/system-session" {
  capabilities = ["read"]
}

# Redis クレデンシャル（セッションストア）
path "secret/data/k1s0/system/redis" {
  capabilities = ["read"]
}
