# featureflag サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、featureflag サービスが必要なパスのみに限定する。
# featureflag サービスはテナントごとのフィーチャーフラグを管理する。
# DB クレデンシャルのみ必要で、Transit や PKI へのアクセスは不要。

# featureflag サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/featureflag/*" {
  capabilities = ["read"]
}

# featureflag シークレットのメタデータ参照
path "secret/metadata/k1s0/system/featureflag/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（featureflag DB のみ）
path "database/creds/system-featureflag" {
  capabilities = ["read"]
}
