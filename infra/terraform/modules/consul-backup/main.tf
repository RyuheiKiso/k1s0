# Consul snapshot バックアップ CronJob
# 毎日実行、7世代保持
# 前提条件: Consul ACL ブートストラップ済みであること（consul acl bootstrap）
# ACLトークンは consul_token_secret_name で指定した Kubernetes Secret から注入される
resource "kubernetes_cron_job_v1" "consul_backup" {
  metadata {
    name      = "consul-backup"
    namespace = var.namespace
    labels = {
      "app.kubernetes.io/name"      = "consul-backup"
      "app.kubernetes.io/component" = "backup"
      "app.kubernetes.io/part-of"   = "k1s0"
    }
  }

  spec {
    schedule                      = var.schedule
    successful_jobs_history_limit = var.retention_count
    failed_jobs_history_limit     = 3
    concurrency_policy            = "Forbid"

    job_template {
      metadata {}
      spec {
        template {
          metadata {}
          spec {
            container {
              name = "consul-backup"
              # Consul スナップショットを取得して PVC（/backup）に保存する。S3 依存なし。
              image = "hashicorp/consul:${var.consul_version}"

              command = ["/bin/sh", "-c"]
              args = [
                <<-EOT
                set -e
                SNAPSHOT_FILE="/backup/consul-snapshot-$(date +%%Y%%m%%d-%%H%%M%%S).snap"
                # Consul Raft スナップショットを取得して PVC に保存する
                consul snapshot save -http-addr=${var.consul_http_addr} "$SNAPSHOT_FILE"
                # ローカルの古いスナップショットを削除（保持世代数を超えた分を削除）
                ls -t /backup/consul-snapshot-*.snap 2>/dev/null | tail -n +$((${var.retention_count} + 1)) | xargs -r rm -f
                EOT
              ]

              volume_mount {
                name       = "backup-volume"
                mount_path = "/backup"
              }

              env {
                name = "CONSUL_HTTP_TOKEN"
                value_from {
                  secret_key_ref {
                    name = var.consul_token_secret_name
                    key  = "token"
                  }
                }
              }
            }

            volume {
              name = "backup-volume"
              persistent_volume_claim {
                claim_name = var.backup_pvc_name
              }
            }

            restart_policy = "OnFailure"
          }
        }
      }
    }
  }
}
