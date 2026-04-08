# CRIT-006 監査対応: vault-server サービス専用 Vault ポリシー
# テナントシークレットの暗号化・管理を担う system ティアのサービス。
# Transit エンジン・vault DB クレデンシャルのみアクセス可能とする。

# vault-server サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/vault/*" {
  capabilities = ["read"]
}

# vault-server シークレットのメタデータ参照
path "secret/metadata/k1s0/system/vault/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（vault DB のみ）
path "database/creds/system-vault" {
  capabilities = ["read"]
}

# Transit エンジン: テナントシークレットの暗号化・復号
path "transit/encrypt/tenant-secrets" {
  capabilities = ["create", "update"]
}

path "transit/decrypt/tenant-secrets" {
  capabilities = ["create", "update"]
}
