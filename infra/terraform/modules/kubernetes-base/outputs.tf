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

output "cluster_roles" {
  description = "Map of created ClusterRole names"
  value = {
    admin     = kubernetes_cluster_role.k1s0_admin.metadata[0].name
    operator  = kubernetes_cluster_role.k1s0_operator.metadata[0].name
    developer = kubernetes_cluster_role.k1s0_developer.metadata[0].name
    readonly  = kubernetes_cluster_role.readonly.metadata[0].name
  }
}
