# C-009 監査対応: PostgreSQL・MySQL バックアップ CronJob（全環境共通）
# 問題: /backup に emptyDir を使用していたため Pod 再起動時にバックアップが消失していた
# 修正: PVC を使用してバックアップを永続化する
# また pg_dump が $PGDATABASE の単一 DB のみ対象だったため全 DB をバックアップするよう修正する

# PostgreSQL バックアップ用 PVC
resource "kubernetes_persistent_volume_claim" "postgresql_backup" {
  count = var.enable_postgresql ? 1 : 0

  metadata {
    name      = "postgresql-backup-pvc"
    namespace = var.database_namespace
    labels = {
      app = "postgresql-backup"
    }
  }

  spec {
    access_modes = ["ReadWriteOnce"]
    resources {
      requests = {
        storage = var.backup_storage_size
      }
    }
    storage_class_name = var.backup_storage_class
  }
}

# PostgreSQL バックアップ CronJob
resource "kubernetes_cron_job_v1" "postgresql_backup" {
  count = var.enable_postgresql ? 1 : 0

  metadata {
    name      = "postgresql-backup"
    namespace = var.database_namespace
  }

  spec {
    schedule = "0 3 * * *"   # 毎日 03:00 UTC

    job_template {
      spec {
        template {
          spec {
            # セキュリティコンテキスト: 非 root 実行・読み取り専用ルートファイルシステム
            security_context {
              run_as_non_root = true
              run_as_user     = 1001
              fs_group        = 1001
              seccomp_profile {
                type = "RuntimeDefault"
              }
            }

            container {
              name = "pg-backup"
              # 全 DB を PVC（/backup）へバックアップする。Pod 再起動でデータ消失しない。
              # M-034 監査対応: k1s0/postgresql-backup はカスタムイメージ（pg_dump 含む）
              # デプロイ前に以下のコマンドでビルド・プッシュが必要:
              #   docker build -t k1s0/postgresql-backup:<version> infra/docker/postgresql-backup/
              #   docker push harbor.internal.example.com/k1s0/postgresql-backup:<version>
              # Dockerfile: infra/docker/postgresql-backup/Dockerfile を参照すること
              image   = "k1s0/postgresql-backup:${var.postgresql_version}"
              command = ["/bin/sh", "-c"]
              # 全サービス DB をループしてバックアップ（K8s版 postgres-backup-cronjob.yaml と同一リスト）
              args    = [<<-EOT
                set -e
                BACKUP_DIR="/backup/postgres"
                TIMESTAMP=$(date +%Y%m%d-%H%M%S)
                mkdir -p "$${BACKUP_DIR}"
                for DB in \
                  k1s0_system auth_db tenant_db session_db config_db featureflag_db \
                  file_db ratelimit_db vault_db quota_db rule_engine_db scheduler_db \
                  search_db api_registry_db app_registry_db service_catalog_db \
                  event_store_db workflow_db policy_db dlq_db \
                  k1s0_saga k1s0_service k1s0_business \
                  notification_db k1s0_event_monitor k1s0_master_maintenance; do
                  echo "Backing up: $${DB}"
                  pg_dump -h "$${PGHOST}" -U "$${PGUSER}" -d "$${DB}" -Fc \
                    -f "$${BACKUP_DIR}/$${DB}-$${TIMESTAMP}.dump"
                done
                find "$${BACKUP_DIR}" -name "*.dump" -mtime +"$${BACKUP_RETENTION_DAYS:-30}" -delete
                echo "All backups completed."
              EOT
              ]

              # コンテナレベルのセキュリティコンテキスト
              security_context {
                read_only_root_filesystem  = true
                allow_privilege_escalation = false
                capabilities {
                  drop = ["ALL"]
                }
              }

              env {
                name  = "PGHOST"
                value = "postgresql.${var.database_namespace}.svc.cluster.local"
              }
              env {
                name  = "BACKUP_RETENTION_DAYS"
                value = "30"
              }
              env_from {
                secret_ref {
                  name = "postgresql-backup-credentials"
                }
              }

              # /backup は PVC からマウント（再起動後もデータ保持）
              volume_mount {
                name       = "backup-volume"
                mount_path = "/backup"
              }
              # /tmp は emptyDir（readOnlyRootFilesystem 対応）
              volume_mount {
                name       = "tmp"
                mount_path = "/tmp"
              }
            }

            # PVC をバックアップストレージとして使用（emptyDir から変更）
            volume {
              name = "backup-volume"
              persistent_volume_claim {
                claim_name = kubernetes_persistent_volume_claim.postgresql_backup[0].metadata[0].name
              }
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

# MySQL バックアップ用 PVC
resource "kubernetes_persistent_volume_claim" "mysql_backup" {
  count = var.enable_mysql ? 1 : 0

  metadata {
    name      = "mysql-backup-pvc"
    namespace = var.database_namespace
    labels = {
      app = "mysql-backup"
    }
  }

  spec {
    access_modes = ["ReadWriteOnce"]
    resources {
      requests = {
        storage = var.backup_storage_size
      }
    }
    storage_class_name = var.backup_storage_class
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
    schedule = "0 3 * * *"   # 毎日 03:00 UTC

    job_template {
      spec {
        template {
          spec {
            # セキュリティコンテキスト: 非 root 実行・読み取り専用ルートファイルシステム
            security_context {
              run_as_non_root = true
              run_as_user     = 1001
              fs_group        = 1001
              seccomp_profile {
                type = "RuntimeDefault"
              }
            }

            container {
              name = "mysql-backup"
              # mysqldump 実行後に PVC（/backup）へ保存する。S3 依存なし。
              image   = "k1s0/mysql-backup:${var.mysql_version}"
              command = ["/bin/sh", "-c"]
              # --defaults-extra-file を使用してパスワードを渡す（プロセスリストへの露出防止）
              args    = [
                "echo '[client]\\nuser='\"$MYSQL_USER\"'\\npassword='\"$MYSQL_PASSWORD\"'\\nhost=mysql' > /tmp/my.cnf && mysqldump --defaults-extra-file=/tmp/my.cnf --all-databases | gzip > /backup/mysql-$(date +%Y%m%d).sql.gz && rm -f /tmp/my.cnf"
              ]

              # コンテナレベルのセキュリティコンテキスト
              security_context {
                read_only_root_filesystem  = true
                allow_privilege_escalation = false
                capabilities {
                  drop = ["ALL"]
                }
              }

              env_from {
                secret_ref {
                  name = "mysql-credentials"
                }
              }

              # /backup は PVC からマウント
              volume_mount {
                name       = "backup-volume"
                mount_path = "/backup"
              }
              volume_mount {
                name       = "tmp"
                mount_path = "/tmp"
              }
            }

            # PVC をバックアップストレージとして使用
            volume {
              name = "backup-volume"
              persistent_volume_claim {
                claim_name = kubernetes_persistent_volume_claim.mysql_backup[0].metadata[0].name
              }
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
