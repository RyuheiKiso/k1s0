variable "strimzi_operator_version" {
  description = "Strimzi Kafka Operator Helm chart version"
  type        = string
}

variable "kafka_broker_replicas" {
  description = "Number of Kafka broker replicas"
  type        = number
  default     = 3
}

# M-19 監査対応: ZooKeeper 変数を削除。KRaft モード移行済み（ADR-0016 参照）。
# zookeeper_replicas と zookeeper_storage_size はここから削除済み。

variable "kafka_default_replication_factor" {
  description = "Default replication factor for Kafka topics"
  type        = number
  default     = 3
}

variable "kafka_min_insync_replicas" {
  description = "Minimum in-sync replicas for Kafka"
  type        = number
  default     = 2
}

variable "kafka_storage_size" {
  description = "Storage size for each Kafka broker"
  type        = string
  default     = "50Gi"
}

variable "kafka_memory_request" {
  description = "Memory request for each Kafka broker"
  type        = string
  default     = "1Gi"
}

variable "kafka_memory_limit" {
  description = "Memory limit for each Kafka broker"
  type        = string
  default     = "2Gi"
}

variable "kafka_cpu_request" {
  description = "CPU request for each Kafka broker"
  type        = string
  default     = "500m"
}

variable "kafka_cpu_limit" {
  description = "CPU limit for each Kafka broker"
  type        = string
  default     = "1000m"
}
