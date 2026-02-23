class EventEnvelope {
  final String eventId;
  final String streamId;
  final int version;
  final String eventType;
  final Object? payload;
  final Object? metadata;
  final DateTime recordedAt;

  const EventEnvelope({
    required this.eventId,
    required this.streamId,
    required this.version,
    required this.eventType,
    this.payload,
    this.metadata,
    required this.recordedAt,
  });
}

class NewEvent {
  final String streamId;
  final String eventType;
  final Object? payload;
  final Object? metadata;

  const NewEvent({
    required this.streamId,
    required this.eventType,
    this.payload,
    this.metadata,
  });
}
