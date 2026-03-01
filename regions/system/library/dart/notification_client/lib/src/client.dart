import 'notification.dart';

abstract class NotificationClient {
  Future<NotificationResponse> send(NotificationRequest request);
  Future<List<NotificationResponse>> sendBatch(List<NotificationRequest> requests);
}

class InMemoryNotificationClient implements NotificationClient {
  final List<NotificationRequest> _sent = [];

  List<NotificationRequest> get sent => List.unmodifiable(_sent);

  @override
  Future<NotificationResponse> send(NotificationRequest request) async {
    _sent.add(request);
    return NotificationResponse(id: request.id, status: 'sent');
  }

  @override
  Future<List<NotificationResponse>> sendBatch(
      List<NotificationRequest> requests) async {
    final responses = <NotificationResponse>[];
    for (final request in requests) {
      responses.add(await send(request));
    }
    return responses;
  }
}
