/// Base interface for all domain events following DDD patterns.
abstract class DomainEvent {
  String get eventType;
  String get aggregateId;
  DateTime get occurredAt;
}

/// Legacy event class for backward compatibility.
/// Extends [DomainEvent] while keeping the original field-based API.
class Event implements DomainEvent {
  final String id;
  @override
  final String eventType;
  @override
  final String aggregateId;
  @override
  final DateTime occurredAt;
  final Map<String, dynamic> payload;
  final DateTime timestamp;

  const Event({
    required this.id,
    required this.eventType,
    required this.payload,
    required this.timestamp,
    this.aggregateId = '',
    DateTime? occurredAt,
  }) : occurredAt = occurredAt ?? timestamp;
}
