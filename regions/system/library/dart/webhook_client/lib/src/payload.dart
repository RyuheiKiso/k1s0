class WebhookPayload {
  final String eventType;
  final String timestamp;
  final Map<String, dynamic> data;

  const WebhookPayload({
    required this.eventType,
    required this.timestamp,
    required this.data,
  });
}
