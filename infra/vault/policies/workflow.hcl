# workflow用Vaultポリシー
# Tier: system
# 説明: workflow固有のシークレットへの読み取りアクセスを提供する

# workflowシークレット
path "secret/data/k1s0/system/workflow/*" {
  capabilities = ["read"]
}

# workflowメタデータ
path "secret/metadata/k1s0/system/workflow/*" {
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
