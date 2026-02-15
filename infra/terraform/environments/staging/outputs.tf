output "namespaces" {
  description = "Created Kubernetes namespaces"
  value       = module.kubernetes_base.namespaces
}

output "storage_classes" {
  description = "Created StorageClass names"
  value = {
    block      = "ceph-block"
    filesystem = "ceph-filesystem"
    block_fast = "ceph-block-fast"
  }
}

output "observability_status" {
  description = "Observability stack deployment status"
  value = {
    prometheus = module.observability.prometheus_status
    loki       = module.observability.loki_status
    jaeger     = module.observability.jaeger_status
  }
}

output "harbor_url" {
  description = "Harbor registry URL"
  value       = module.harbor.harbor_url
}
