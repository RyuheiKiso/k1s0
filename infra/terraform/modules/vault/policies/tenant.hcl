# tenant サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、tenant サービスが必要なパスのみに限定する。
# tenant サービスはマルチテナント管理を担い、テナント固有の設定値の読み取りが必要。
# Kafka SASL クレデンシャルはテナント分離イベント発行のために必要。

# tenant サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/tenant/*" {
  capabilities = ["read"]
}

# tenant シークレットのメタデータ参照
path "secret/metadata/k1s0/system/tenant/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（tenant DB のみ）
path "database/creds/system-tenant" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（テナント変更イベント発行のため）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
