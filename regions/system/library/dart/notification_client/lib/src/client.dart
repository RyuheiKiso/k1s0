import 'notification.dart';

abstract class NotificationClient {
  Future<NotificationResponse> send(NotificationRequest request);
}

class InMemoryNotificationClient implements NotificationClient {
  final List<NotificationRequest> _sent = [];

  List<NotificationRequest> get sent => List.unmodifiable(_sent);

  @override
  Future<NotificationResponse> send(NotificationRequest request) async {
    _sent.add(request);
    return NotificationResponse(id: request.id, status: 'sent');
  }
}
