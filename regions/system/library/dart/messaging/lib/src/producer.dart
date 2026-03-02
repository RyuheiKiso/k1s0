import 'types.dart';

abstract class EventProducer {
  Future<void> publish(EventEnvelope event);
  Future<void> publishBatch(List<EventEnvelope> events);
  Future<void> close();
}

class NoOpEventProducer implements EventProducer {
  final List<EventEnvelope> published = [];

  @override
  Future<void> publish(EventEnvelope event) async {
    published.add(event);
  }

  @override
  Future<void> publishBatch(List<EventEnvelope> events) async {
    published.addAll(events);
  }

  @override
  Future<void> close() async {}
}
