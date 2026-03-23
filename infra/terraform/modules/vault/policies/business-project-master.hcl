# project-master ドメイン専用 Vault ポリシー（I-5 対応）
# business tier 内でドメイン単位のシークレット分離を実現する

path "secret/data/k1s0/business/project-master/*" {
  capabilities = ["read", "list"]
}

path "database/creds/business-project-master" {
  capabilities = ["read"]
}

# Kafka SASL クレデンシャルは業務上 system tier の共有リソース
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
