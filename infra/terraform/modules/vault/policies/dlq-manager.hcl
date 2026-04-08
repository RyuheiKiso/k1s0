# CRIT-006 監査対応: dlq-manager サービス専用 Vault ポリシー
# Dead Letter Queue の管理・再処理を担う system ティアのサービス。
# DLQ DB クレデンシャル・Kafka SASL のみアクセス可能とする。

# dlq-manager サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/dlq-manager/*" {
  capabilities = ["read"]
}

# dlq-manager シークレットのメタデータ参照
path "secret/metadata/k1s0/system/dlq-manager/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（dlq-manager DB のみ）
path "database/creds/system-dlq-manager" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（DLQ メッセージ再処理に必要）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
