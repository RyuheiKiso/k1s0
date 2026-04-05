# event-monitor サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、event-monitor サービスが必要なパスのみに限定する。
# event-monitor サービスはイベントストリームの監視・アラートを担う。
# Kafka 購読（全イベントトピック監視）と DB クレデンシャルが必要。

# event-monitor サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/event-monitor/*" {
  capabilities = ["read"]
}

# event-monitor シークレットのメタデータ参照
path "secret/metadata/k1s0/system/event-monitor/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（event-monitor DB のみ）
path "database/creds/system-event-monitor" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（イベントストリーム購読・監視のため）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
