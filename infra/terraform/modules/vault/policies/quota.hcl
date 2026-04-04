# quota サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、quota サービスが必要なパスのみに限定する。
# quota サービスはテナントごとのリソース使用量制限を管理する。
# DB クレデンシャルと Redis（カウンタキャッシュ）クレデンシャルが必要。

# quota サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/quota/*" {
  capabilities = ["read"]
}

# quota シークレットのメタデータ参照
path "secret/metadata/k1s0/system/quota/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（quota DB のみ）
path "database/creds/system-quota" {
  capabilities = ["read"]
}

# Redis クレデンシャル（クォータカウンタキャッシュ）
path "secret/data/k1s0/system/redis" {
  capabilities = ["read"]
}
