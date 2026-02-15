output "vault_addr" {
  description = "Vault server address"
  value       = var.vault_address
}

output "vault_token_path" {
  description = "Path to the Vault token file used by Vault Agent on Kubernetes pods"
  value       = "/vault/secrets/token"
}

output "secret_paths" {
  description = "Map of Vault secret engine mount paths"
  value = {
    kv       = vault_mount.kv.path
    database = vault_mount.database.path
    pki      = vault_mount.pki.path
  }
}
