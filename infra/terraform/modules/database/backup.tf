# PostgreSQL バックアップ CronJob
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
            # セキュリティコンテキスト: 非 root 実行・読み取り専用ルートファイルシステム
            security_context {
              run_as_non_root = true
              run_as_user     = 1001
              fs_group        = 1001
            }

            container {
              name    = "pg-backup"
              image   = "bitnami/postgresql:${var.postgresql_version}"
              command = ["/bin/sh", "-c"]
              args    = [
                "pg_dump -h postgresql -U $PGUSER -d $PGDATABASE | gzip > /backup/pg-$(date +%Y%m%d).sql.gz && s3cmd put /backup/pg-$(date +%Y%m%d).sql.gz s3://${var.backup_bucket}/postgresql/"
              ]

              # コンテナレベルのセキュリティコンテキスト
              security_context {
                read_only_root_filesystem = true
                allow_privilege_escalation = false
              }

              env_from {
                secret_ref {
                  name = "postgresql-credentials"
                }
              }

              # /backup と /tmp は書き込みが必要なため emptyDir をマウント
              volume_mount {
                name       = "backup-tmp"
                mount_path = "/backup"
              }
              volume_mount {
                name       = "tmp"
                mount_path = "/tmp"
              }
            }

            # 書き込み用の一時ボリューム
            volume {
              name = "backup-tmp"
              empty_dir {}
            }
            volume {
              name = "tmp"
              empty_dir {}
            }

            restart_policy = "OnFailure"
          }
        }
      }
    }
  }
}

# MySQL バックアップ CronJob
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
            # セキュリティコンテキスト: 非 root 実行・読み取り専用ルートファイルシステム
            security_context {
              run_as_non_root = true
              run_as_user     = 1001
              fs_group        = 1001
            }

            container {
              name    = "mysql-backup"
              image   = "bitnami/mysql:${var.mysql_version}"
              command = ["/bin/sh", "-c"]
              # --defaults-extra-file を使用してパスワードを渡す
              # プロセスリストにパスワードが露出するのを防止する
              args    = [
                "echo '[client]\nuser='\"$MYSQL_USER\"'\npassword='\"$MYSQL_PASSWORD\"'\nhost=mysql' > /tmp/my.cnf && mysqldump --defaults-extra-file=/tmp/my.cnf --all-databases | gzip > /backup/mysql-$(date +%Y%m%d).sql.gz && rm -f /tmp/my.cnf && s3cmd put /backup/mysql-$(date +%Y%m%d).sql.gz s3://${var.backup_bucket}/mysql/"
              ]

              # コンテナレベルのセキュリティコンテキスト
              security_context {
                read_only_root_filesystem = true
                allow_privilege_escalation = false
              }

              env_from {
                secret_ref {
                  name = "mysql-credentials"
                }
              }

              # /backup と /tmp は書き込みが必要なため emptyDir をマウント
              volume_mount {
                name       = "backup-tmp"
                mount_path = "/backup"
              }
              volume_mount {
                name       = "tmp"
                mount_path = "/tmp"
              }
            }

            # 書き込み用の一時ボリューム
            volume {
              name = "backup-tmp"
              empty_dir {}
            }
            volume {
              name = "tmp"
              empty_dir {}
            }

            restart_policy = "OnFailure"
          }
        }
      }
    }
  }
}
