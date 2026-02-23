enum NotificationChannel { email, sms, push, webhook }

class NotificationRequest {
  final String id;
  final NotificationChannel channel;
  final String recipient;
  final String? subject;
  final String body;

  const NotificationRequest({
    required this.id,
    required this.channel,
    required this.recipient,
    this.subject,
    required this.body,
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
