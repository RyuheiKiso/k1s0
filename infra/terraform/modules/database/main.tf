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

# --- Backup CronJobs ---

resource "kubernetes_cron_job_v1" "postgresql_backup" {
  count = var.enable_postgresql ? 1 : 0

  metadata {
    name      = "postgresql-backup"
    namespace = var.database_namespace
  }

  spec {
    schedule = "0 2 * * *"   # 毎日 02:00 JST

    job_template {
      spec {
        template {
          spec {
            container {
              name    = "pg-backup"
              image   = "bitnami/postgresql:${var.postgresql_version}"
              command = ["/bin/sh", "-c"]
              args    = [
                "pg_dump -h postgresql -U $PGUSER -d $PGDATABASE | gzip > /backup/pg-$(date +%Y%m%d).sql.gz && s3cmd put /backup/pg-$(date +%Y%m%d).sql.gz s3://${var.backup_bucket}/postgresql/"
              ]

              env_from {
                secret_ref {
                  name = "postgresql-credentials"
                }
              }
            }

            restart_policy = "OnFailure"
          }
        }
      }
    }
  }
}

resource "kubernetes_cron_job_v1" "mysql_backup" {
  count = var.enable_mysql ? 1 : 0

  metadata {
    name      = "mysql-backup"
    namespace = var.database_namespace
  }

  spec {
    schedule = "0 2 * * *"   # 毎日 02:00 JST

    job_template {
      spec {
        template {
          spec {
            container {
              name    = "mysql-backup"
              image   = "bitnami/mysql:${var.mysql_version}"
              command = ["/bin/sh", "-c"]
              args    = [
                "mysqldump -h mysql -u $MYSQL_USER -p$MYSQL_PASSWORD --all-databases | gzip > /backup/mysql-$(date +%Y%m%d).sql.gz && s3cmd put /backup/mysql-$(date +%Y%m%d).sql.gz s3://${var.backup_bucket}/mysql/"
              ]

              env_from {
                secret_ref {
                  name = "mysql-credentials"
                }
              }
            }

            restart_policy = "OnFailure"
          }
        }
      }
    }
  }
}
