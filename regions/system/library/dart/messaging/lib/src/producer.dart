import 'types.dart';

abstract class EventProducer {
  Future<void> publish(EventEnvelope event);
  Future<void> close();
}

class NoOpEventProducer implements EventProducer {
  final List<EventEnvelope> published = [];

  @override
  Future<void> publish(EventEnvelope event) async {
    published.add(event);
  }

  @override
  Future<void> close() async {}
}
