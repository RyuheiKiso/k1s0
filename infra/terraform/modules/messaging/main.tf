# L-12 監査対応: Terraform provider バージョン制約を追加する。
# バージョン制約がないと、terraform init 時に最新バージョンが選択されて破壊的変更が入る可能性がある。
# ~> 演算子でマイナーバージョンアップを許容しつつメジャーバージョン固定とする。
terraform {
  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
  }
}

# M-6 監査対応: Strimzi Operator の replicas 設定に本番運用コメントを追加する。
# 現在シングルレプリカ（replicas=1）で動作しているが、本番環境では SPOF（単一障害点）となる。
# 本番環境では replicas を 3 以上に設定すること（SPOF 回避）。
# 開発/ステージング環境ではコスト削減のため 1 を許容する。
resource "helm_release" "strimzi_operator" {
  name       = "strimzi-kafka-operator"
  namespace  = "messaging"
  repository = "https://strimzi.io/charts/"
  chart      = "strimzi-kafka-operator"
  version    = var.strimzi_operator_version

  create_namespace = true

  set {
    # M-6 監査対応: 本番環境では replicas を 3 以上に設定すること（SPOF 回避）
    # 現在の値 "1" は開発/ステージング環境専用。本番では var.strimzi_operator_replicas 等で上書きすること。
    name  = "replicas"
    value = "1"
  }
}

# Kafka cluster deployed as a Kubernetes manifest via the Strimzi CRD.
# The Strimzi operator watches for Kafka CRs and manages the cluster lifecycle.
resource "kubernetes_manifest" "kafka_cluster" {
  manifest = {
    apiVersion = "kafka.strimzi.io/v1beta2"
    kind       = "Kafka"
    metadata = {
      name      = "k1s0-kafka"
      namespace = "messaging"
    }
    spec = {
      kafka = {
        version  = "3.6.1"
        replicas = var.kafka_broker_replicas
        # M-7 監査対応: plain（非暗号化）リスナーを廃止し TLS リスナーのみにする。
        # ゼロトラストアーキテクチャの観点から内部通信も暗号化する。
        # docker-compose 開発環境は PLAINTEXT を継続使用（本番 K8s 環境のみ TLS 強制）。
        # 参照: ADR-0016（Kafka KRaft移行）
        listeners = [
          {
            name = "tls"
            port = 9093
            type = "internal"
            tls  = true
          }
        ]
        config = {
          "offsets.topic.replication.factor"         = var.kafka_default_replication_factor
          "transaction.state.log.replication.factor" = var.kafka_default_replication_factor
          "transaction.state.log.min.isr"            = var.kafka_min_insync_replicas
          "default.replication.factor"               = var.kafka_default_replication_factor
          "min.insync.replicas"                      = var.kafka_min_insync_replicas
          "log.retention.hours"                      = 168
        }
        storage = {
          type = "persistent-claim"
          size = var.kafka_storage_size
          class = "ceph-block-fast"
        }
        resources = {
          requests = {
            memory = var.kafka_memory_request
            cpu    = var.kafka_cpu_request
          }
          limits = {
            memory = var.kafka_memory_limit
            cpu    = var.kafka_cpu_limit
          }
        }
      }
      # M-19 監査対応: KRaft モード移行済み（ADR-0016 参照）。ZooKeeper 設定を削除。
      # Strimzi v0.29 以降は KRaft モードをサポートし、ZooKeeper は不要となった。
      # zookeeper ブロックはここから削除済み（ADR-0016: kafka-kraft-migration で廃止）。
      entityOperator = {
        topicOperator = {}
        userOperator  = {}
      }
    }
  }

  depends_on = [helm_release.strimzi_operator]
}
