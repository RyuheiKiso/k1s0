# search用Vaultポリシー
# Tier: system
# 説明: search固有のシークレットへの読み取りアクセスを提供する

# searchシークレット
path "secret/data/k1s0/system/search/*" {
  capabilities = ["read"]
}

# searchメタデータ
path "secret/metadata/k1s0/system/search/*" {
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

# MEDIUM-INFRA-001 監査対応: システムリース更新権限を追加する
# 長時間稼働するサービスでリース期限切れによる接続断を防止する
path "sys/leases/renew" {
  capabilities = ["update"]
}

# 自身のトークン情報確認と更新のための権限
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}
