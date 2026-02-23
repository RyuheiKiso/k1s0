class Event {
  final String id;
  final String eventType;
  final Map<String, dynamic> payload;
  final DateTime timestamp;

  const Event({
    required this.id,
    required this.eventType,
    required this.payload,
    required this.timestamp,
  });
}
