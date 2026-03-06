enum NotificationChannel { email, sms, push, slack, webhook }

class NotificationRequest {
  final String id;
  final NotificationChannel channel;
  final String recipient;
  final String? subject;
  final String body;
  final Map<String, dynamic>? metadata;

  const NotificationRequest({
    required this.id,
    required this.channel,
    required this.recipient,
    this.subject,
    required this.body,
    this.metadata,
  });
}

class NotificationResponse {
  final String id;
  final String status;
  final String? messageId;

  const NotificationResponse({
    required this.id,
    required this.status,
    this.messageId,
  });
}
