variable "strimzi_operator_version" {
  description = "Strimzi Kafka Operator Helm chart version"
  type        = string
}

variable "kafka_broker_replicas" {
  description = "Number of Kafka broker replicas"
  type        = number
  default     = 3
}

variable "zookeeper_replicas" {
  description = "Number of ZooKeeper replicas"
  type        = number
  default     = 3
}

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

variable "zookeeper_storage_size" {
  description = "Storage size for each ZooKeeper node"
  type        = string
  default     = "10Gi"
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
