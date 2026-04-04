# api-registry サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、api-registry サービスが必要なパスのみに限定する。
# api-registry サービスは API スキーマ・バージョン管理を担う。
# DB クレデンシャルのみ必要で、Kafka・Transit・PKI へのアクセスは不要。

# api-registry サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/api-registry/*" {
  capabilities = ["read"]
}

# api-registry シークレットのメタデータ参照
path "secret/metadata/k1s0/system/api-registry/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（api-registry DB のみ）
path "database/creds/system-api-registry" {
  capabilities = ["read"]
}
