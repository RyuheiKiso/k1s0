# tenant用Vaultポリシー
# Tier: system
# 説明: tenant固有のシークレットへの読み取りアクセスを提供する

# tenantシークレット
path "secret/data/k1s0/system/tenant/*" {
  capabilities = ["read"]
}

# tenantメタデータ
path "secret/metadata/k1s0/system/tenant/*" {
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
