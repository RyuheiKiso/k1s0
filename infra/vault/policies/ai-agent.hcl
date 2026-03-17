# ai-agent用Vaultポリシー
# Tier: system
# 説明: ai-agent固有のシークレットへの読み取りアクセスを提供する

# ai-agentシークレット
path "secret/data/k1s0/system/ai-agent/*" {
  capabilities = ["read"]
}

# ai-agentメタデータ
path "secret/metadata/k1s0/system/ai-agent/*" {
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
