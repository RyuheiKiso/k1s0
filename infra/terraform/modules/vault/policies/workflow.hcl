# workflow サービス専用 Vault ポリシー（H-010 監査対応）
# 最小権限原則に従い、workflow サービスが必要なパスのみに限定する。
# workflow サービスはワークフロー定義・インスタンス・タスクの管理を担う。
# Kafka イベント発行（ワークフロー状態変更通知）と DB クレデンシャルが必要。

# workflow サービス固有の KV v2 シークレット
path "secret/data/k1s0/system/workflow/*" {
  capabilities = ["read"]
}

# workflow シークレットのメタデータ参照
path "secret/metadata/k1s0/system/workflow/*" {
  capabilities = ["read", "list"]
}

# DB 動的クレデンシャル（workflow DB のみ）
path "database/creds/system-workflow" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャル（ワークフローイベント発行のため）
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
