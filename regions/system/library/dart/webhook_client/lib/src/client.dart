import 'payload.dart';

abstract class WebhookClient {
  Future<int> send(String url, WebhookPayload payload);
}

class InMemoryWebhookClient implements WebhookClient {
  final List<(String, WebhookPayload)> _sent = [];

  List<(String, WebhookPayload)> get sent => List.unmodifiable(_sent);

  @override
  Future<int> send(String url, WebhookPayload payload) async {
    _sent.add((url, payload));
    return 200;
  }
}
