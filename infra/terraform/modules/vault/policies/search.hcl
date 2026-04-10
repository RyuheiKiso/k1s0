# CRIT-006 監査対応: search サービス専用 Vault ポリシー
# 全文検索・インデックス管理を担う system ティアのサービス。
# 検索 DB クレデンシャル・OpenSearch 認証情報のみアクセス可能とする。

# search サービス固有の KV v2 シークレット（OpenSearch 認証情報等）
path "secret/data/k1s0/system/search/*" {
  capabilities = ["read"]
}

# search シークレットのメタデータ参照
path "secret/metadata/k1s0/system/search/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（search DB のみ）
path "database/creds/system-search" {
  capabilities = ["read"]
}
