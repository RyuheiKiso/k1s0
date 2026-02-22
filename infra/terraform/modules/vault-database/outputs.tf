# Vault Database Module - Outputs

output "database_mount_path" {
  description = "Vault database secret engine mount path"
  value       = vault_mount.database.path
}

output "role_names" {
  description = "Map of service names to their Vault database role names"
  value = {
    auth_server_rw    = vault_database_secret_backend_role.auth_server_rw.name
    auth_server_ro    = vault_database_secret_backend_role.auth_server_ro.name
    config_server_rw  = vault_database_secret_backend_role.config_server_rw.name
    config_server_ro  = vault_database_secret_backend_role.config_server_ro.name
    saga_server_rw    = vault_database_secret_backend_role.saga_server_rw.name
    saga_server_ro    = vault_database_secret_backend_role.saga_server_ro.name
    dlq_manager_rw    = vault_database_secret_backend_role.dlq_manager_rw.name
    dlq_manager_ro    = vault_database_secret_backend_role.dlq_manager_ro.name
  }
}
