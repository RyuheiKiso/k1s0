# prod 環境の出力値
# modules/environment の出力を中継する

output "namespaces" {
  description = "作成された Kubernetes Namespace の一覧"
  value       = module.environment.namespaces
}

output "storage_classes" {
  description = "作成された StorageClass 名"
  value       = module.environment.storage_classes
}

output "observability_status" {
  description = "Observability スタックのデプロイ状態"
  value       = module.environment.observability_status
}

output "harbor_url" {
  description = "Harbor レジストリ URL"
  value       = module.environment.harbor_url
}
