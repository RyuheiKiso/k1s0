# =============================================================================
# infra/security/openbao/policies/ci-runner.hcl
#
# CI runner（self-hosted ARC、リリース時点+ で導入）が OpenBao からアクセス可能な path。
# 設計: docs/05_実装/85_Identity設計/30_OpenBao/secrets-matrix.md
#       docs/05_実装/30_CI_CD設計/10_reusable_workflow/01_reusable_workflow設計.md
#
# リリース時点 では GitHub-hosted runner + GH Actions encrypted secrets 経由で
# secret を扱うため本 policy の発動なし。リリース時点+ で self-hosted ARC を導入した
# 段階で適用する。
# =============================================================================

# CI が必要とする限定 secret 集合（ci/ prefix で隔離）
path "secret/data/k1s0/ci/*" {
  capabilities = ["read"]
}

# Harbor robot account / GHCR mirror 認証情報（air-gapped 環境）
path "secret/data/k1s0/ci/registry/*" {
  capabilities = ["read"]
}

# Renovate token（依存更新自動化）
path "secret/data/k1s0/ci/renovate/*" {
  capabilities = ["read"]
}

# tier1 / tier2 / tier3 の prod secret には絶対触れない
path "secret/data/k1s0/prod/*" {
  capabilities = ["deny"]
}

path "transit/*" {
  capabilities = ["deny"]
}
path "sys/*" {
  capabilities = ["deny"]
}
