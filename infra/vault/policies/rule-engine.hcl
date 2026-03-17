# rule-engine用Vaultポリシー
# Tier: system
# 説明: rule-engine固有のシークレットへの読み取りアクセスを提供する

# rule-engineシークレット
path "secret/data/k1s0/system/rule-engine/*" {
  capabilities = ["read"]
}

# rule-engineメタデータ
path "secret/metadata/k1s0/system/rule-engine/*" {
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
