resource "helm_release" "postgresql" {
  count      = var.enable_postgresql ? 1 : 0
  name       = "postgresql"
  namespace  = var.database_namespace
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "postgresql"
  version    = var.postgresql_chart_version

  values = [file("${path.module}/values/postgresql-${var.environment}.yaml")]
}

resource "helm_release" "mysql" {
  count      = var.enable_mysql ? 1 : 0
  name       = "mysql"
  namespace  = var.database_namespace
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "mysql"
  version    = var.mysql_chart_version

  values = [file("${path.module}/values/mysql-${var.environment}.yaml")]
}
