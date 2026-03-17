# vault-server用Vaultポリシー
# Tier: system
# 説明: vault-server固有のシークレットへの読み取りアクセスを提供する

# vault-serverシークレット
path "secret/data/k1s0/system/vault-server/*" {
  capabilities = ["read"]
}

# vault-serverメタデータ
path "secret/metadata/k1s0/system/vault-server/*" {
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
