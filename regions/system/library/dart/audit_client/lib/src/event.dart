class AuditEvent {
  final String id;
  final String tenantId;
  final String actorId;
  final String action;
  final String resourceType;
  final String resourceId;
  final DateTime timestamp;

  const AuditEvent({
    required this.id,
    required this.tenantId,
    required this.actorId,
    required this.action,
    required this.resourceType,
    required this.resourceId,
    required this.timestamp,
  });
}
