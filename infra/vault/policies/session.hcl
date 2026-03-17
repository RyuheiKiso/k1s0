# session用Vaultポリシー
# Tier: system
# 説明: session固有のシークレットへの読み取りアクセスを提供する

# sessionシークレット
path "secret/data/k1s0/system/session/*" {
  capabilities = ["read"]
}

# sessionメタデータ
path "secret/metadata/k1s0/system/session/*" {
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
