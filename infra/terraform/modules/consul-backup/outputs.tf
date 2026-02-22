output "cron_job_name" {
  description = "Name of the Consul backup CronJob"
  value       = kubernetes_cron_job_v1.consul_backup.metadata[0].name
}

output "cron_job_namespace" {
  description = "Namespace of the Consul backup CronJob"
  value       = kubernetes_cron_job_v1.consul_backup.metadata[0].namespace
}

output "schedule" {
  description = "Cron schedule of the backup job"
  value       = var.schedule
}
