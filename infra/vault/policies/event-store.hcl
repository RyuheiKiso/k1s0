# event-store用Vaultポリシー
# Tier: system
# 説明: event-store固有のシークレットへの読み取りアクセスを提供する

# event-storeシークレット
path "secret/data/k1s0/system/event-store/*" {
  capabilities = ["read"]
}

# event-storeメタデータ
path "secret/metadata/k1s0/system/event-store/*" {
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
