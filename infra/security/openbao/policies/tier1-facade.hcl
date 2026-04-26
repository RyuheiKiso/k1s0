# =============================================================================
# infra/security/openbao/policies/tier1-facade.hcl
#
# tier1 facade（Go ファサード + Rust core）が OpenBao からアクセス可能な path を定義する policy。
# 設計: docs/05_実装/85_Identity設計/30_OpenBao/secrets-matrix.md
# 関連: ADR-SEC-002（OpenBao）
#
# 認証経路:
#   採用側 prod: Kubernetes ServiceAccount JWT auth または SPIFFE ID auth
#   dev:        root token（dev mode）
#
# 重要: 本 hcl 内の `{{environment}}` / `{{tenant_id}}` は Vault Templated Policies の
#       正規構文（`{{identity.entity.metadata.X}}`）ではない。採用側で envsubst / sed で
#       実値に展開してから `bao policy write` する**前段テンプレ**として扱う。
#       Vault Templated Policies に切替える場合は `{{identity.entity.metadata.tenant_id}}`
#       のように entity metadata からの解決に書き直すこと（Vault 公式 doc 参照）。
#
# 適用方法（採用側 prod 想定）:
#   ENV=prod TENANT=acme envsubst < tier1-facade.hcl > /tmp/tier1-facade.expanded.hcl
#   # （または: sed -e "s/{{environment}}/$ENV/g" -e "s/{{tenant_id}}/$TENANT/g"）
#   bao policy write tier1-facade /tmp/tier1-facade.expanded.hcl
#   bao write auth/kubernetes/role/tier1-facade \
#       bound_service_account_names=tier1-facade \
#       bound_service_account_namespaces=k1s0-tier1 \
#       policies=tier1-facade
# =============================================================================

# -----------------------------------------------------------------------------
# tier1 facade 自身の secret（DB / Kafka / OIDC client / 等）
# -----------------------------------------------------------------------------
path "secret/data/k1s0/{{environment}}/tier1/*" {
  capabilities = ["read", "list"]
}

path "secret/metadata/k1s0/{{environment}}/tier1/*" {
  capabilities = ["read", "list"]
}

# -----------------------------------------------------------------------------
# Audit chain / Decision 用の限定書き込み（Rust core からのみ）
# -----------------------------------------------------------------------------
# Audit chain 自体は Postgres に保管するが、暗号化に使う envelope key は OpenBao Transit。
path "transit/encrypt/k1s0-audit-{{environment}}" {
  capabilities = ["update"]
}
path "transit/decrypt/k1s0-audit-{{environment}}" {
  capabilities = ["update"]
}

# -----------------------------------------------------------------------------
# tenant 別 secret（multi-tenancy 境界、plan 04-17 と整合）
# tier1 facade は principal の tenant_id に従い、対応 path のみアクセス。
# -----------------------------------------------------------------------------
# 注: 動的な tenant_id 制限は OpenBao 標準の Sentinel / EGP では tenant ごとの
#     policy 自動生成が必要。リリース時点 では tier1 facade コード側で明示制御し、
#     OpenBao 側は「全 tenant 配下を読める」状態とする。
path "secret/data/k1s0/{{environment}}/tenants/+/tier1/*" {
  capabilities = ["read", "list"]
}

# -----------------------------------------------------------------------------
# 禁止 path（明示的 deny、root token 越し操作の予防）
# -----------------------------------------------------------------------------
# tier2 / tier3 の secret には触れない（最小権限）
path "secret/data/k1s0/{{environment}}/tier2/*" {
  capabilities = ["deny"]
}
path "secret/data/k1s0/{{environment}}/tier3/*" {
  capabilities = ["deny"]
}
path "sys/*" {
  capabilities = ["deny"]
}
