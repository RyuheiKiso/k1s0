# service-catalog用Vaultポリシー
# Tier: system
# 説明: service-catalog固有のシークレットへの読み取りアクセスを提供する

# service-catalogシークレット
path "secret/data/k1s0/system/service-catalog/*" {
  capabilities = ["read"]
}

# service-catalogメタデータ
path "secret/metadata/k1s0/system/service-catalog/*" {
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
