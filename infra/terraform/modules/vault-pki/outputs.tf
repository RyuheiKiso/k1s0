# Vault PKI Module - Outputs

output "root_ca_cert" {
  description = "Root CA certificate (PEM)"
  value       = vault_pki_secret_backend_root_cert.root.certificate
}

output "intermediate_ca_cert" {
  description = "Intermediate CA certificate (PEM)"
  value       = vault_pki_secret_backend_root_sign_intermediate.intermediate.certificate
}

output "pki_int_mount_path" {
  description = "Intermediate PKI mount path for certificate issuance"
  value       = vault_mount.pki_int.path
}

output "role_names" {
  description = "Map of tier names to PKI role names"
  value = {
    system   = vault_pki_secret_backend_role.system.name
    business = vault_pki_secret_backend_role.business.name
    service  = vault_pki_secret_backend_role.service.name
  }
}
