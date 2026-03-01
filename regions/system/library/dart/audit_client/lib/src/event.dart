import 'package:uuid/uuid.dart';

class AuditEvent {
  final String id;
  final String tenantId;
  final String actorId;
  final String action;
  final String resourceType;
  final String resourceId;
  final Map<String, dynamic>? metadata;
  final DateTime timestamp;

  AuditEvent({
    String? id,
    required this.tenantId,
    required this.actorId,
    required this.action,
    required this.resourceType,
    required this.resourceId,
    this.metadata,
    DateTime? timestamp,
  })  : id = id ?? const Uuid().v4(),
        timestamp = timestamp ?? DateTime.now();
}
