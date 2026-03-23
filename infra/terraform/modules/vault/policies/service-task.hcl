# task ドメイン専用 Vault ポリシー（I-5 対応）
# service tier 内で task ドメインのシークレットを他ドメインから分離する
path "secret/data/k1s0/service/task/*" {
  capabilities = ["read", "list"]
}

path "database/creds/service-task" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャルは業務上 system tier の共有リソース
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
