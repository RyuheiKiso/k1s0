import 'event.dart';

typedef EventHandler = Future<void> Function(Event event);

class InMemoryEventBus {
  final Map<String, List<EventHandler>> _handlers = {};

  void subscribe(String eventType, EventHandler handler) {
    (_handlers[eventType] ??= []).add(handler);
  }

  void unsubscribe(String eventType) => _handlers.remove(eventType);

  Future<void> publish(Event event) async {
    final handlers = _handlers[event.eventType] ?? [];
    for (final h in handlers) {
      await h(event);
    }
  }
}
