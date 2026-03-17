# graphql-gateway用Vaultポリシー
# Tier: system
# 説明: graphql-gateway固有のシークレットへの読み取りアクセスを提供する

# graphql-gatewayシークレット
path "secret/data/k1s0/system/graphql-gateway/*" {
  capabilities = ["read"]
}

# graphql-gatewayメタデータ
path "secret/metadata/k1s0/system/graphql-gateway/*" {
  capabilities = ["read", "list"]
}

# 共有データベース認証情報
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}

# 共有Kafka認証情報
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}
