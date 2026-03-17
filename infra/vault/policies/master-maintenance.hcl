# master-maintenance用Vaultポリシー
# Tier: system
# 説明: master-maintenance固有のシークレットへの読み取りアクセスを提供する

# master-maintenanceシークレット
path "secret/data/k1s0/system/master-maintenance/*" {
  capabilities = ["read"]
}

# master-maintenanceメタデータ
path "secret/metadata/k1s0/system/master-maintenance/*" {
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
