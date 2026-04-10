# CRIT-006 監査対応: config-server サービス専用 Vault ポリシー
# テナント設定値の管理・配信を担う system ティアのサービス。
# 設定 DB クレデンシャルのみアクセス可能とする。

# config-server サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/config/*" {
  capabilities = ["read"]
}

# config-server シークレットのメタデータ参照
path "secret/metadata/k1s0/system/config/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（config DB のみ）
path "database/creds/system-config" {
  capabilities = ["read"]
}
