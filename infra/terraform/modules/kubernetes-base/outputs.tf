output "namespaces" {
  description = "Map of created namespace names to their tier labels"
  value = {
    for k, v in kubernetes_namespace.tier : k => {
      name = v.metadata[0].name
      tier = v.metadata[0].labels["tier"]
    }
  }
}

output "namespace_names" {
  description = "List of created namespace names"
  value       = [for k, v in kubernetes_namespace.tier : v.metadata[0].name]
}
