# CRIT-006 監査対応: event-store サービス専用 Vault ポリシー
# イベントソーシング・CQRS パターンのイベントストアを担う system ティアのサービス。
# イベント DB クレデンシャル・Kafka SASL のみアクセス可能とする。

# event-store サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/event-store/*" {
  capabilities = ["read"]
}

# event-store シークレットのメタデータ参照
path "secret/metadata/k1s0/system/event-store/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（event-store DB のみ）
path "database/creds/system-event-store" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（イベント配信に必要）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
