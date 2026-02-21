import 'types.dart';

typedef EventHandler = Future<void> Function(EventEnvelope event);

abstract class EventConsumer {
  Future<void> subscribe(String topic, EventHandler handler);
  Future<void> close();
}
