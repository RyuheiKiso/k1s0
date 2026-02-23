import 'package:test/test.dart';

import 'package:k1s0_event_bus/event_bus.dart';

void main() {
  late InMemoryEventBus bus;

  setUp(() {
    bus = InMemoryEventBus();
  });

  Event _makeEvent(String type) => Event(
        id: 'evt-1',
        eventType: type,
        payload: {'key': 'value'},
        timestamp: DateTime.now(),
      );

  group('subscribe and publish', () {
    test('handler receives matching event', () async {
      final received = <Event>[];
      bus.subscribe('user.created', (e) async => received.add(e));
      await bus.publish(_makeEvent('user.created'));
      expect(received, hasLength(1));
      expect(received.first.eventType, equals('user.created'));
    });

    test('handler does not receive non-matching event', () async {
      final received = <Event>[];
      bus.subscribe('user.created', (e) async => received.add(e));
      await bus.publish(_makeEvent('user.deleted'));
      expect(received, isEmpty);
    });

    test('multiple handlers for same event', () async {
      var count = 0;
      bus.subscribe('test', (e) async => count++);
      bus.subscribe('test', (e) async => count++);
      await bus.publish(_makeEvent('test'));
      expect(count, equals(2));
    });
  });

  group('unsubscribe', () {
    test('removes all handlers for event type', () async {
      final received = <Event>[];
      bus.subscribe('test', (e) async => received.add(e));
      bus.unsubscribe('test');
      await bus.publish(_makeEvent('test'));
      expect(received, isEmpty);
    });
  });

  group('Event', () {
    test('stores all fields', () {
      final event = _makeEvent('test');
      expect(event.id, equals('evt-1'));
      expect(event.eventType, equals('test'));
      expect(event.payload, containsPair('key', 'value'));
    });
  });
}
