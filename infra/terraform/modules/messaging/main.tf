resource "helm_release" "strimzi_operator" {
  name       = "strimzi-kafka-operator"
  namespace  = "messaging"
  repository = "https://strimzi.io/charts/"
  chart      = "strimzi-kafka-operator"
  version    = var.strimzi_operator_version

  create_namespace = true

  set {
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
        listeners = [
          {
            name = "plain"
            port = 9092
            type = "internal"
            tls  = false
          },
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
      zookeeper = {
        replicas = var.zookeeper_replicas
        storage = {
          type = "persistent-claim"
          size = var.zookeeper_storage_size
          class = "ceph-block"
        }
        resources = {
          requests = {
            memory = "512Mi"
            cpu    = "250m"
          }
          limits = {
            memory = "1Gi"
            cpu    = "500m"
          }
        }
      }
      entityOperator = {
        topicOperator = {}
        userOperator  = {}
      }
    }
  }

  depends_on = [helm_release.strimzi_operator]
}
