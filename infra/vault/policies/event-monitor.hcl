# event-monitor用Vaultポリシー
# Tier: system
# 説明: event-monitor固有のシークレットへの読み取りアクセスを提供する

# event-monitorシークレット
path "secret/data/k1s0/system/event-monitor/*" {
  capabilities = ["read"]
}

# event-monitorメタデータ
path "secret/metadata/k1s0/system/event-monitor/*" {
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
