import 'package:uuid/uuid.dart';

const _uuid = Uuid();

class EventMetadata {
  final String eventId;
  final String eventType;
  final String correlationId;
  final String traceId;
  final DateTime timestamp;
  final String source;
  final int schemaVersion;

  const EventMetadata({
    required this.eventId,
    required this.eventType,
    required this.correlationId,
    required this.traceId,
    required this.timestamp,
    required this.source,
    this.schemaVersion = 1,
  });

  factory EventMetadata.create(
    String eventType,
    String source, {
    String? correlationId,
    String? traceId,
  }) {
    return EventMetadata(
      eventId: _uuid.v4(),
      eventType: eventType,
      correlationId: correlationId ?? _uuid.v4(),
      traceId: traceId ?? _uuid.v4(),
      timestamp: DateTime.now().toUtc(),
      source: source,
      schemaVersion: 1,
    );
  }
}

class EventEnvelope {
  final String topic;
  final String key;
  final Object payload;
  final EventMetadata metadata;

  const EventEnvelope({
    required this.topic,
    required this.key,
    required this.payload,
    required this.metadata,
  });
}
