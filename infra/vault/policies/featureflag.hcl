# featureflag用Vaultポリシー
# Tier: system
# 説明: featureflag固有のシークレットへの読み取りアクセスを提供する

# featureflagシークレット
path "secret/data/k1s0/system/featureflag/*" {
  capabilities = ["read"]
}

# featureflagメタデータ
path "secret/metadata/k1s0/system/featureflag/*" {
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
