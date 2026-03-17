# ratelimit用Vaultポリシー
# Tier: system
# 説明: ratelimit固有のシークレットへの読み取りアクセスを提供する

# ratelimitシークレット
path "secret/data/k1s0/system/ratelimit/*" {
  capabilities = ["read"]
}

# ratelimitメタデータ
path "secret/metadata/k1s0/system/ratelimit/*" {
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
