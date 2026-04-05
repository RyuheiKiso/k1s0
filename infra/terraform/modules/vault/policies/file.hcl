# file サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、file サービスが必要なパスのみに限定する。
# file サービスはファイルのアップロード・ダウンロード・管理を担う。
# オブジェクトストレージ（S3 等）クレデンシャルと DB クレデンシャルが必要。

# file サービス固有の KV v2 シークレット（S3 アクセスキー、バケット名等）
path "secret/data/k1s0/system/file/*" {
  capabilities = ["read"]
}

# file シークレットのメタデータ参照
path "secret/metadata/k1s0/system/file/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（file DB のみ）
path "database/creds/system-file" {
  capabilities = ["read"]
}
