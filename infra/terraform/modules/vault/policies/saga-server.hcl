# CRIT-006 監査対応: saga-server サービス専用 Vault ポリシー
# Saga オーケストレーションパターンによる分散トランザクション管理を担う system ティアのサービス。
# Saga DB クレデンシャル・Kafka SASL のみアクセス可能とする。

# saga-server サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/saga/*" {
  capabilities = ["read"]
}

# saga-server シークレットのメタデータ参照
path "secret/metadata/k1s0/system/saga/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（saga DB のみ）
path "database/creds/system-saga" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（補償トランザクション配信に必要）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
