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
      test('ハンドラーが一致するイベントを受信すること', () async {
        final received = <Event>[];
        bus.subscribe('user.created', (e) async => received.add(e));
        await bus.publish(makeEvent('user.created'));
        expect(received, hasLength(1));
        expect(received.first.eventType, equals('user.created'));
      });

      test('ハンドラーが一致しないイベントを受信しないこと', () async {
        final received = <Event>[];
        bus.subscribe('user.created', (e) async => received.add(e));
        await bus.publish(makeEvent('user.deleted'));
        expect(received, isEmpty);
      });

      test('同じイベントに複数のハンドラーを登録できること', () async {
        var count = 0;
        bus.subscribe('test', (e) async => count++);
        bus.subscribe('test', (e) async => count++);
        await bus.publish(makeEvent('test'));
        expect(count, equals(2));
      });
    });

    group('unsubscribe', () {
      test('イベントタイプの全ハンドラーが削除されること', () async {
        final received = <Event>[];
        bus.subscribe('test', (e) async => received.add(e));
        bus.unsubscribe('test');
        await bus.publish(makeEvent('test'));
        expect(received, isEmpty);
      });
    });

    group('Event', () {
      test('全フィールドが正しく保存されること', () {
        final event = makeEvent('test');
        expect(event.id, equals('evt-1'));
        expect(event.eventType, equals('test'));
        expect(event.payload, containsPair('key', 'value'));
      });

      test('DomainEventインターフェースを実装していること', () {
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
      test('サブスクライブされたハンドラーが一致するイベントを受信すること', () async {
        final handler = CollectingHandler<DomainEvent>();
        bus.subscribe('user.created', handler);
        final event = TestDomainEvent('user.created');
        await bus.publish(event);
        expect(handler.received, hasLength(1));
        expect(handler.received.first, same(event));
      });

      test('ハンドラーが一致しないイベントを受信しないこと', () async {
        final handler = CollectingHandler<DomainEvent>();
        bus.subscribe('user.created', handler);
        await bus.publish(TestDomainEvent('order.placed'));
        expect(handler.received, isEmpty);
      });

      test('同じイベントタイプに複数のハンドラーを登録できること', () async {
        final h1 = CollectingHandler<DomainEvent>();
        final h2 = CollectingHandler<DomainEvent>();
        bus.subscribe('test', h1);
        bus.subscribe('test', h2);
        await bus.publish(TestDomainEvent('test'));
        expect(h1.received, hasLength(1));
        expect(h2.received, hasLength(1));
      });

      test('ハンドラーが登録されていない場合にパブリッシュしてもエラーにならないこと', () async {
        await bus.publish(TestDomainEvent('unknown'));
      });
    });

    group('EventSubscription', () {
      test('subscribeがEventSubscriptionを返すこと', () {
        final handler = CollectingHandler<DomainEvent>();
        final sub = bus.subscribe('test', handler);
        expect(sub.eventType, equals('test'));
        expect(sub.isActive, isTrue);
      });

      test('unsubscribeでハンドラーが削除されること', () async {
        final handler = CollectingHandler<DomainEvent>();
        final sub = bus.subscribe('test', handler);
        sub.unsubscribe();
        expect(sub.isActive, isFalse);
        await bus.publish(TestDomainEvent('test'));
        expect(handler.received, isEmpty);
      });

      test('unsubscribeで特定のハンドラーのみ削除されること', () async {
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
      test('デフォルト設定で正常に動作すること', () async {
        final defaultBus = EventBus();
        final handler = CollectingHandler<DomainEvent>();
        defaultBus.subscribe('test', handler);
        await defaultBus.publish(TestDomainEvent('test'));
        expect(handler.received, hasLength(1));
      });

      test('カスタム設定で正常に動作すること', () async {
        final customBus =
            EventBus(const EventBusConfig(bufferSize: 100, handlerTimeoutMs: 1000));
        final handler = CollectingHandler<DomainEvent>();
        customBus.subscribe('test', handler);
        await customBus.publish(TestDomainEvent('test'));
        expect(handler.received, hasLength(1));
      });

      test('ハンドラーのタイムアウト時にHANDLER_FAILEDエラーがスローされること', () async {
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
      test('ハンドラーがスローした場合にHANDLER_FAILEDエラーが発生すること', () async {
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

      test('クローズ後にパブリッシュするとCHANNEL_CLOSEDエラーが発生すること', () {
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

      test('クローズ後にサブスクライブするとCHANNEL_CLOSEDエラーが発生すること', () {
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

      test('エラーのプロパティが正しい値を持つこと', () {
        const err = EventBusError(
          'test message',
          EventBusErrorCode.publishFailed,
        );
        expect(err.message, equals('test message'));
        expect(err.code, equals(EventBusErrorCode.publishFailed));
        expect(err.codeString, equals('PUBLISH_FAILED'));
        expect(err.toString(), contains('PUBLISH_FAILED'));
      });

      test('全エラーコードが正しい文字列表現を持つこと', () {
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
      test('TestDomainEventが正しいプロパティを持つこと', () {
        final event = TestDomainEvent('user.created', aggregateId: 'user-123');
        expect(event.eventType, equals('user.created'));
        expect(event.aggregateId, equals('user-123'));
        expect(event.occurredAt, isA<DateTime>());
      });
    });
  });
}
