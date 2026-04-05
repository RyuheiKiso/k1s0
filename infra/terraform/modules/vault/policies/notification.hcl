# notification サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、notification サービスが必要なパスのみに限定する。
# notification サービスはメール・プッシュ通知の送信を担う。
# 外部通知プロバイダ（SMTP、FCM 等）のクレデンシャルと DB クレデンシャルが必要。

# notification サービス固有の KV v2 シークレット（SMTP 設定、FCM キー等）
path "secret/data/k1s0/system/notification/*" {
  capabilities = ["read"]
}

# notification シークレットのメタデータ参照
path "secret/metadata/k1s0/system/notification/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（notification DB のみ）
path "database/creds/system-notification" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（通知イベント購読のため）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
