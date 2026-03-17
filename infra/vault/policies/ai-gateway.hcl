# ai-gateway用Vaultポリシー
# Tier: system
# 説明: ai-gateway固有のシークレットへの読み取りアクセスを提供する

# ai-gatewayシークレット
path "secret/data/k1s0/system/ai-gateway/*" {
  capabilities = ["read"]
}

# ai-gatewayメタデータ
path "secret/metadata/k1s0/system/ai-gateway/*" {
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
