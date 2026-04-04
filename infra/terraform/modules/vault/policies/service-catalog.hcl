# service-catalog サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、service-catalog サービスが必要なパスのみに限定する。
# service-catalog サービスはサービス定義・エンドポイントカタログの管理を担う。
# DB クレデンシャルのみ必要で、Kafka・Transit・PKI へのアクセスは不要。

# service-catalog サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/service-catalog/*" {
  capabilities = ["read"]
}

# service-catalog シークレットのメタデータ参照
path "secret/metadata/k1s0/system/service-catalog/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（service-catalog DB のみ）
path "database/creds/system-service-catalog" {
  capabilities = ["read"]
}
