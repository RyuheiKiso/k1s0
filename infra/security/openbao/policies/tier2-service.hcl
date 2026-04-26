# =============================================================================
# infra/security/openbao/policies/tier2-service.hcl
#
# tier2 ドメイン共通サービス（C# / Go）が OpenBao からアクセス可能な path を定義する policy。
# 設計: docs/05_実装/85_Identity設計/30_OpenBao/secrets-matrix.md
# 関連: ADR-SEC-002（OpenBao）/ ADR-TIER1-003（言語不可視）
#
# tier2 は原則 tier1 の Secret API 経由でアクセスするが、本 policy はその裏付けとして
# OpenBao 側に「tier2 から直接アクセス可能な path」を限定的に定義する。
# =============================================================================

# tier2 サービス自身の運用 secret（外部 API キー / SaaS token 等）
path "secret/data/k1s0/{{environment}}/tier2/+/*" {
  capabilities = ["read"]
}

# 自分の tenant 配下のみ
path "secret/data/k1s0/{{environment}}/tenants/{{tenant_id}}/tier2/*" {
  capabilities = ["read"]
}

# tier1 / tier3 の secret には触れない
path "secret/data/k1s0/{{environment}}/tier1/*" {
  capabilities = ["deny"]
}
path "secret/data/k1s0/{{environment}}/tier3/*" {
  capabilities = ["deny"]
}
path "secret/data/k1s0/{{environment}}/tenants/+/tier1/*" {
  capabilities = ["deny"]
}

# Transit は tier2 から直接呼ばず、tier1 Crypto API 経由
path "transit/*" {
  capabilities = ["deny"]
}

path "sys/*" {
  capabilities = ["deny"]
}
