# Realm-level roles
resource "keycloak_role" "sys_admin" {
  realm_id    = keycloak_realm.k1s0.id
  name        = "sys_admin"
  description = "System administrator with full access to all resources"
}

resource "keycloak_role" "sys_operator" {
  realm_id    = keycloak_realm.k1s0.id
  name        = "sys_operator"
  description = "System operator for monitoring, log access, and configuration"
}

resource "keycloak_role" "sys_auditor" {
  realm_id    = keycloak_realm.k1s0.id
  name        = "sys_auditor"
  description = "Auditor with read-only access to all resources"
}

resource "keycloak_role" "user" {
  realm_id    = keycloak_realm.k1s0.id
  name        = "user"
  description = "Standard authenticated user"
}
