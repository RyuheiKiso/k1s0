# app-registry サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、app-registry サービスが必要なパスのみに限定する。
# app-registry サービスはアプリケーション登録・設定管理を担う。
# DB クレデンシャルのみ必要で、Kafka・Transit・PKI へのアクセスは不要。

# app-registry サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/app-registry/*" {
  capabilities = ["read"]
}

# app-registry シークレットのメタデータ参照
path "secret/metadata/k1s0/system/app-registry/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（app-registry DB のみ）
path "database/creds/system-app-registry" {
  capabilities = ["read"]
}
