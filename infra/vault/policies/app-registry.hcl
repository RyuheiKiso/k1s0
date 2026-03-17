# app-registry用Vaultポリシー
# Tier: system
# 説明: app-registry固有のシークレットへの読み取りアクセスを提供する

# app-registryシークレット
path "secret/data/k1s0/system/app-registry/*" {
  capabilities = ["read"]
}

# app-registryメタデータ
path "secret/metadata/k1s0/system/app-registry/*" {
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
