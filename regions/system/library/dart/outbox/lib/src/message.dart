import 'package:uuid/uuid.dart';

const _uuid = Uuid();

enum OutboxStatus { pending, processing, delivered, failed }

class OutboxMessage {
  final String id;
  final String topic;
  final String eventType;
  final String payload;
  final OutboxStatus status;
  final int retryCount;
  final DateTime scheduledAt;
  final DateTime createdAt;
  final DateTime updatedAt;
  final String correlationId;

  const OutboxMessage({
    required this.id,
    required this.topic,
    required this.eventType,
    required this.payload,
    required this.status,
    required this.retryCount,
    required this.scheduledAt,
    required this.createdAt,
    required this.updatedAt,
    required this.correlationId,
  });

  OutboxMessage copyWith({
    String? id,
    String? topic,
    String? eventType,
    String? payload,
    OutboxStatus? status,
    int? retryCount,
    DateTime? scheduledAt,
    DateTime? createdAt,
    DateTime? updatedAt,
    String? correlationId,
  }) {
    return OutboxMessage(
      id: id ?? this.id,
      topic: topic ?? this.topic,
      eventType: eventType ?? this.eventType,
      payload: payload ?? this.payload,
      status: status ?? this.status,
      retryCount: retryCount ?? this.retryCount,
      scheduledAt: scheduledAt ?? this.scheduledAt,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
      correlationId: correlationId ?? this.correlationId,
    );
  }
}

OutboxMessage createOutboxMessage(
  String topic,
  String eventType,
  String payload,
  String correlationId,
) {
  final now = DateTime.now().toUtc();
  return OutboxMessage(
    id: _uuid.v4(),
    topic: topic,
    eventType: eventType,
    payload: payload,
    status: OutboxStatus.pending,
    retryCount: 0,
    scheduledAt: now,
    createdAt: now,
    updatedAt: now,
    correlationId: correlationId,
  );
}

DateTime nextScheduledAt(int retryCount) {
  var delayMinutes = 1 << retryCount;
  if (delayMinutes > 60) delayMinutes = 60;
  return DateTime.now().toUtc().add(Duration(minutes: delayMinutes));
}

bool canTransitionTo(OutboxStatus from, OutboxStatus to) {
  switch (from) {
    case OutboxStatus.pending:
      return to == OutboxStatus.processing;
    case OutboxStatus.processing:
      return to == OutboxStatus.delivered || to == OutboxStatus.failed;
    case OutboxStatus.failed:
      return to == OutboxStatus.pending;
    case OutboxStatus.delivered:
      return false;
  }
}
