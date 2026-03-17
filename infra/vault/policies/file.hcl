# file用Vaultポリシー
# Tier: system
# 説明: file固有のシークレットへの読み取りアクセスを提供する

# fileシークレット
path "secret/data/k1s0/system/file/*" {
  capabilities = ["read"]
}

# fileメタデータ
path "secret/metadata/k1s0/system/file/*" {
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
