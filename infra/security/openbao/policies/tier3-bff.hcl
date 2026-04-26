# =============================================================================
# infra/security/openbao/policies/tier3-bff.hcl
#
# tier3 BFF / Web / Native が OpenBao からアクセス可能な path を定義する policy。
# 設計: docs/05_実装/85_Identity設計/30_OpenBao/secrets-matrix.md
#
# tier3 は原則 BFF 経由で tier1 Secret API を叩く。OpenBao への直接アクセスは
# BFF の起動時設定（OIDC client secret 等）に限定する。
# =============================================================================

# tier3 BFF の起動 secret（OIDC / session encryption key 等）
path "secret/data/k1s0/{{environment}}/tier3/bff/*" {
  capabilities = ["read"]
}

# 自分の tenant 配下のみ
path "secret/data/k1s0/{{environment}}/tenants/{{tenant_id}}/tier3/*" {
  capabilities = ["read"]
}

# 他 tier には触れない
path "secret/data/k1s0/{{environment}}/tier1/*" {
  capabilities = ["deny"]
}
path "secret/data/k1s0/{{environment}}/tier2/*" {
  capabilities = ["deny"]
}

path "transit/*" {
  capabilities = ["deny"]
}
path "sys/*" {
  capabilities = ["deny"]
}
