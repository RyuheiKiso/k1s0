import 'dart:typed_data';
import 'component.dart';

class Message {
  final String topic;
  final Uint8List data;
  final Map<String, String> metadata;
  final String id;

  const Message({
    required this.topic,
    required this.data,
    required this.metadata,
    required this.id,
  });
}

abstract class MessageHandler {
  Future<void> handle(Message message);
}

abstract class PubSub implements Component {
  Future<void> publish(String topic, Uint8List data, {Map<String, String>? metadata});
  Future<String> subscribe(String topic, MessageHandler handler);
  Future<void> unsubscribe(String subscriptionId);
}
