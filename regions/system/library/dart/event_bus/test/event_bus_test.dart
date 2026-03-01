import 'package:test/test.dart';

import 'package:k1s0_event_bus/event_bus.dart';

// ---------- Test helpers ----------

Event makeEvent(String type) => Event(
      id: 'evt-1',
      eventType: type,
      payload: {'key': 'value'},
      timestamp: DateTime.now(),
      aggregateId: 'agg-1',
    );

/// Simple DomainEvent implementation for testing.
class TestDomainEvent implements DomainEvent {
  @override
  final String eventType;
  @override
  final String aggregateId;
  @override
  final DateTime occurredAt;

  TestDomainEvent(this.eventType, {this.aggregateId = 'agg-1'})
      : occurredAt = DateTime.now();
}

/// Collecting handler for tests.
class CollectingHandler<T extends DomainEvent> implements EventHandler<T> {
  final List<T> received = [];

  @override
  Future<void> handle(T event) async {
    received.add(event);
  }
}

/// Handler that throws on invocation.
class FailingHandler<T extends DomainEvent> implements EventHandler<T> {
  @override
  Future<void> handle(T event) async {
    throw Exception('boom');
  }
}

/// Handler that delays longer than the timeout.
class SlowHandler<T extends DomainEvent> implements EventHandler<T> {
  final Duration delay;
  SlowHandler(this.delay);

  @override
  Future<void> handle(T event) async {
    await Future.delayed(delay);
  }
}

// ---------- Legacy InMemoryEventBus tests ----------

void main() {
  group('InMemoryEventBus (legacy)', () {
    late InMemoryEventBus bus;

    setUp(() {
      bus = InMemoryEventBus();
    });

    group('subscribe and publish', () {
      test('handler receives matching event', () async {
        final received = <Event>[];
        bus.subscribe('user.created', (e) async => received.add(e));
        await bus.publish(makeEvent('user.created'));
        expect(received, hasLength(1));
        expect(received.first.eventType, equals('user.created'));
      });

      test('handler does not receive non-matching event', () async {
        final received = <Event>[];
        bus.subscribe('user.created', (e) async => received.add(e));
        await bus.publish(makeEvent('user.deleted'));
        expect(received, isEmpty);
      });

      test('multiple handlers for same event', () async {
        var count = 0;
        bus.subscribe('test', (e) async => count++);
        bus.subscribe('test', (e) async => count++);
        await bus.publish(makeEvent('test'));
        expect(count, equals(2));
      });
    });

    group('unsubscribe', () {
      test('removes all handlers for event type', () async {
        final received = <Event>[];
        bus.subscribe('test', (e) async => received.add(e));
        bus.unsubscribe('test');
        await bus.publish(makeEvent('test'));
        expect(received, isEmpty);
      });
    });

    group('Event', () {
      test('stores all fields', () {
        final event = makeEvent('test');
        expect(event.id, equals('evt-1'));
        expect(event.eventType, equals('test'));
        expect(event.payload, containsPair('key', 'value'));
      });

      test('implements DomainEvent', () {
        final event = makeEvent('test');
        expect(event, isA<DomainEvent>());
        expect(event.aggregateId, equals('agg-1'));
        expect(event.occurredAt, isA<DateTime>());
      });
    });
  });

  // ---------- DDD EventBus tests ----------

  group('EventBus', () {
    late EventBus bus;

    setUp(() {
      bus = EventBus();
    });

    group('publish / subscribe', () {
      test('subscribed handler receives matching event', () async {
        final handler = CollectingHandler<DomainEvent>();
        bus.subscribe('user.created', handler);
        final event = TestDomainEvent('user.created');
        await bus.publish(event);
        expect(handler.received, hasLength(1));
        expect(handler.received.first, same(event));
      });

      test('handler does not receive non-matching event', () async {
        final handler = CollectingHandler<DomainEvent>();
        bus.subscribe('user.created', handler);
        await bus.publish(TestDomainEvent('order.placed'));
        expect(handler.received, isEmpty);
      });

      test('multiple handlers for same event type', () async {
        final h1 = CollectingHandler<DomainEvent>();
        final h2 = CollectingHandler<DomainEvent>();
        bus.subscribe('test', h1);
        bus.subscribe('test', h2);
        await bus.publish(TestDomainEvent('test'));
        expect(h1.received, hasLength(1));
        expect(h2.received, hasLength(1));
      });

      test('publish with no handlers does not throw', () async {
        await bus.publish(TestDomainEvent('unknown'));
      });
    });

    group('EventSubscription', () {
      test('subscribe returns EventSubscription', () {
        final handler = CollectingHandler<DomainEvent>();
        final sub = bus.subscribe('test', handler);
        expect(sub.eventType, equals('test'));
        expect(sub.isActive, isTrue);
      });

      test('unsubscribe removes handler', () async {
        final handler = CollectingHandler<DomainEvent>();
        final sub = bus.subscribe('test', handler);
        sub.unsubscribe();
        expect(sub.isActive, isFalse);
        await bus.publish(TestDomainEvent('test'));
        expect(handler.received, isEmpty);
      });

      test('unsubscribe removes only the specific handler', () async {
        final h1 = CollectingHandler<DomainEvent>();
        final h2 = CollectingHandler<DomainEvent>();
        final sub1 = bus.subscribe('test', h1);
        bus.subscribe('test', h2);
        sub1.unsubscribe();
        await bus.publish(TestDomainEvent('test'));
        expect(h1.received, isEmpty);
        expect(h2.received, hasLength(1));
      });
    });

    group('EventBusConfig', () {
      test('works with default config', () async {
        final defaultBus = EventBus();
        final handler = CollectingHandler<DomainEvent>();
        defaultBus.subscribe('test', handler);
        await defaultBus.publish(TestDomainEvent('test'));
        expect(handler.received, hasLength(1));
      });

      test('works with custom config', () async {
        final customBus =
            EventBus(const EventBusConfig(bufferSize: 100, handlerTimeoutMs: 1000));
        final handler = CollectingHandler<DomainEvent>();
        customBus.subscribe('test', handler);
        await customBus.publish(TestDomainEvent('test'));
        expect(handler.received, hasLength(1));
      });

      test('handler timeout throws HANDLER_FAILED', () async {
        final slowBus =
            EventBus(const EventBusConfig(handlerTimeoutMs: 50));
        slowBus.subscribe<DomainEvent>(
          'test',
          SlowHandler(const Duration(milliseconds: 200)),
        );
        expect(
          () => slowBus.publish(TestDomainEvent('test')),
          throwsA(isA<EventBusError>().having(
            (e) => e.code,
            'code',
            EventBusErrorCode.handlerFailed,
          )),
        );
      });
    });

    group('EventBusError', () {
      test('HANDLER_FAILED when handler throws', () async {
        bus.subscribe<DomainEvent>('test', FailingHandler());
        expect(
          () => bus.publish(TestDomainEvent('test')),
          throwsA(isA<EventBusError>().having(
            (e) => e.code,
            'code',
            EventBusErrorCode.handlerFailed,
          )),
        );
      });

      test('CHANNEL_CLOSED on publish after close', () {
        bus.close();
        expect(
          () => bus.publish(TestDomainEvent('test')),
          throwsA(isA<EventBusError>().having(
            (e) => e.code,
            'code',
            EventBusErrorCode.channelClosed,
          )),
        );
      });

      test('CHANNEL_CLOSED on subscribe after close', () {
        bus.close();
        expect(
          () => bus.subscribe('test', CollectingHandler<DomainEvent>()),
          throwsA(isA<EventBusError>().having(
            (e) => e.code,
            'code',
            EventBusErrorCode.channelClosed,
          )),
        );
      });

      test('error properties are correct', () {
        const err = EventBusError(
          'test message',
          EventBusErrorCode.publishFailed,
        );
        expect(err.message, equals('test message'));
        expect(err.code, equals(EventBusErrorCode.publishFailed));
        expect(err.codeString, equals('PUBLISH_FAILED'));
        expect(err.toString(), contains('PUBLISH_FAILED'));
      });

      test('all error codes have correct string representation', () {
        expect(
          const EventBusError('', EventBusErrorCode.publishFailed).codeString,
          equals('PUBLISH_FAILED'),
        );
        expect(
          const EventBusError('', EventBusErrorCode.handlerFailed).codeString,
          equals('HANDLER_FAILED'),
        );
        expect(
          const EventBusError('', EventBusErrorCode.channelClosed).codeString,
          equals('CHANNEL_CLOSED'),
        );
      });
    });

    group('DomainEvent', () {
      test('TestDomainEvent has correct properties', () {
        final event = TestDomainEvent('user.created', aggregateId: 'user-123');
        expect(event.eventType, equals('user.created'));
        expect(event.aggregateId, equals('user-123'));
        expect(event.occurredAt, isA<DateTime>());
      });
    });
  });
}
