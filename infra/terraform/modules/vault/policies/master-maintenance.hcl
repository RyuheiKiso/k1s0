# CRIT-006 監査対応: master-maintenance サービス専用 Vault ポリシー
# マスターデータのメンテナンス・バッチ処理を担う system ティアのサービス。
# 複数 DB への読み書きが必要なため、system 共通シークレットにもアクセスできる。

# master-maintenance サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/master-maintenance/*" {
  capabilities = ["read"]
}

# master-maintenance シークレットのメタデータ参照
path "secret/metadata/k1s0/system/master-maintenance/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（master-maintenance DB のみ）
path "database/creds/system-master-maintenance" {
  capabilities = ["read"]
}
