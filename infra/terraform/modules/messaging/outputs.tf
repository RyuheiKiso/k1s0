output "kafka_bootstrap_servers" {
  description = "Kafka bootstrap server addresses for internal cluster access"
  value       = "k1s0-kafka-kafka-bootstrap.messaging.svc.cluster.local:9092"
}

output "schema_registry_url" {
  description = "Schema Registry URL for Protobuf/Avro schema management"
  value       = "http://schema-registry.k1s0-system.svc.cluster.local:8081"
}

output "topic_names" {
  description = "Map of Kafka topic names by domain"
  value = {
    system_auth_login        = "k1s0.system.auth.login.v1"
    system_audit_events      = "k1s0.system.audit.events.v1"
    # タスク管理トピック
    service_task_created     = "k1s0.service.task.created.v1"
    service_task_updated     = "k1s0.service.task.updated.v1"
    service_task_cancelled   = "k1s0.service.task.cancelled.v1"
    service_board_column_updated = "k1s0.service.board.column_updated.v1"
    service_activity_created = "k1s0.service.activity.created.v1"
    service_activity_approved = "k1s0.service.activity.approved.v1"
    business_taskmanagement_project_type_changed = "k1s0.business.taskmanagement.projectmaster.project_type_changed.v1"
    business_taskmanagement_status_definition_changed = "k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1"
  }
}
