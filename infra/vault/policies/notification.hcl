# notification用Vaultポリシー
# Tier: system
# 説明: notification固有のシークレットへの読み取りアクセスを提供する

# notificationシークレット
path "secret/data/k1s0/system/notification/*" {
  capabilities = ["read"]
}

# notificationメタデータ
path "secret/metadata/k1s0/system/notification/*" {
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
