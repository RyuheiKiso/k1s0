output "vault_addr" {
  description = "Vault server address"
  value       = var.vault_address
}

output "vault_token_path" {
  description = "Path to the Vault token file used by Vault Agent on Kubernetes pods"
  value       = "/vault/secrets/token"
}

# Vault シークレットエンジンのマウントパスをまとめて出力する
# database と pki はサブモジュール (vault-database, vault-pki) が canonical owner のため固定値を使用
output "secret_paths" {
  description = "Map of Vault secret engine mount paths"
  value = {
    kv       = vault_mount.kv.path
    database = "database"
    pki      = "pki"
  }
}
