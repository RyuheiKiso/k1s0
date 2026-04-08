# CRIT-006 監査対応: navigation サービス専用 Vault ポリシー
# テナントナビゲーション定義の管理を担う system ティアのサービス。
# ナビゲーション DB クレデンシャルのみアクセス可能とする。

# navigation サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/navigation/*" {
  capabilities = ["read"]
}

# navigation シークレットのメタデータ参照
path "secret/metadata/k1s0/system/navigation/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（navigation DB のみ）
path "database/creds/system-navigation" {
  capabilities = ["read"]
}
