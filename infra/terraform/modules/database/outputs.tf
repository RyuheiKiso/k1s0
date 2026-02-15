output "postgresql_host" {
  description = "PostgreSQL service hostname"
  value       = var.enable_postgresql ? "postgresql.${var.database_namespace}.svc.cluster.local" : null
}

output "postgresql_port" {
  description = "PostgreSQL service port"
  value       = var.enable_postgresql ? 5432 : null
}

output "mysql_host" {
  description = "MySQL service hostname"
  value       = var.enable_mysql ? "mysql.${var.database_namespace}.svc.cluster.local" : null
}

output "mysql_port" {
  description = "MySQL service port"
  value       = var.enable_mysql ? 3306 : null
}
