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
    service_order_created    = "k1s0.service.order.created.v1"
    service_order_updated    = "k1s0.service.order.updated.v1"
    service_inventory_reserved = "k1s0.service.inventory.reserved.v1"
    business_accounting_entry = "k1s0.business.accounting.entry.v1"
  }
}
