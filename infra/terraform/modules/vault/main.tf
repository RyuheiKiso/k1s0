# Vault Module - Main Configuration
# Manages HashiCorp Vault setup for the k1s0 project.
# Includes secret engines, audit logging, and base configuration.

terraform {
  required_providers {
    vault = {
      source  = "hashicorp/vault"
      version = "~> 4.0"
    }
  }
}

# ============================================================
# Secret Engines
# ============================================================

# KV v2 Secret Engine - Static secrets (API keys, config values, etc.)
# NOTE: database mount と pki mount はサブモジュール (vault-database, vault-pki) が canonical owner
# NOTE: kubernetes auth backend と roles は auth.tf が canonical owner
resource "vault_mount" "kv" {
  path        = "secret"
  type        = "kv-v2"
  description = "KV v2 secret engine for static secrets"
}

# ============================================================
# Audit Configuration
# ============================================================

# M-031 監査対応: 監査ログを PVC に永続化する（Pod 再起動でログが消失しないよう）
# コンプライアンス要件: 監査ログは永続化が必要
#
# 実装済みリソース:
#   - PVC:       infra/kubernetes/system/vault-audit-pvc.yaml（kubectl kustomize 経由で適用）
#   - Helm 設定: infra/terraform/modules/vault/values/vault-helm-overrides.yaml
#   - PVC サイズ変数: var.vault_audit_storage_size（デフォルト 10Gi）
#
# デプロイ手順:
#   1. kubectl kustomize infra/kubernetes/system/ | kubectl apply -f -
#   2. helm upgrade vault hashicorp/vault \
#        -f infra/terraform/modules/vault/values/vault-helm-overrides.yaml \
#        -n k1s0-system
#
# Audit log - records all authentication attempts, secret reads,
# policy changes, and configuration changes.
# Secret values are masked (log_raw = false).
resource "vault_audit" "file" {
  type = "file"
  options = {
    file_path = "/vault/logs/audit.log"
    log_raw   = "false"
  }
}
