# rule-engine サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、rule-engine サービスが必要なパスのみに限定する。
# rule-engine サービスはビジネスルールの評価・管理を担う。
# Kafka イベント発行（ルール評価結果通知）と DB クレデンシャルが必要。

# rule-engine サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/rule-engine/*" {
  capabilities = ["read"]
}

# rule-engine シークレットのメタデータ参照
path "secret/metadata/k1s0/system/rule-engine/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（rule-engine DB のみ）
path "database/creds/system-rule-engine" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（ルール評価イベント発行のため）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
