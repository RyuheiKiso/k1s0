# 環境共通モジュールの出力値
# 各環境の outputs.tf からこのモジュールの出力を参照する

output "namespaces" {
  description = "作成された Kubernetes Namespace の一覧"
  value       = module.kubernetes_base.namespaces
}

output "storage_classes" {
  description = "作成された StorageClass 名"
  value = {
    block      = "ceph-block"
    filesystem = "ceph-filesystem"
    block_fast = "ceph-block-fast"
  }
}

output "observability_status" {
  description = "Observability スタックのデプロイ状態"
  value = {
    prometheus = module.observability.prometheus_status
    loki       = module.observability.loki_status
    jaeger     = module.observability.jaeger_status
  }
}

output "harbor_url" {
  description = "Harbor レジストリ URL"
  value       = module.harbor.harbor_url
}
