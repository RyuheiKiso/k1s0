# policy サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、policy サービスが必要なパスのみに限定する。
# policy サービスはアクセス制御ポリシー（RBAC/ABAC）の管理を担う。
# DB クレデンシャルのみ必要で、Transit・PKI へのアクセスは不要。

# policy サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/policy/*" {
  capabilities = ["read"]
}

# policy シークレットのメタデータ参照
path "secret/metadata/k1s0/system/policy/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（policy DB のみ）
path "database/creds/system-policy" {
  capabilities = ["read"]
}
