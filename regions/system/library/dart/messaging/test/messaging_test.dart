import 'package:test/test.dart';
import 'package:k1s0_messaging/messaging.dart';

void main() {
  group('EventMetadata', () {
    test('createでUUID形式のeventIdが生成されること', () {
      final meta = EventMetadata.create(
        'user.created',
        'auth-service',
        correlationId: 'corr-100',
      );
      expect(meta.eventId, isNotEmpty);
      final uuidRegex = RegExp(
          r'^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$');
      expect(uuidRegex.hasMatch(meta.eventId), isTrue);
    });

    test('createで指定したcorrelationIdが使用されること', () {
      final meta =
          EventMetadata.create('event', 'svc', correlationId: 'corr-123');
      expect(meta.correlationId, equals('corr-123'));
    });

    test('createで指定したtraceIdが使用されること', () {
      final meta =
          EventMetadata.create('event', 'svc', correlationId: 'corr-124', traceId: 'trace-123');
      expect(meta.traceId, equals('trace-123'));
    });

    test('createでtraceIdが自動生成されること', () {
      final meta = EventMetadata.create('event', 'svc', correlationId: 'corr-125');
      expect(meta.correlationId, equals('corr-125'));
      expect(meta.traceId, isNotEmpty);
    });

    test('createでタイムスタンプがUTCに設定されること', () {
      final meta = EventMetadata.create('event', 'svc', correlationId: 'corr-126');
      expect(meta.timestamp.isUtc, isTrue);
    });

    test('createでschemaVersionのデフォルト値が1になること', () {
      final meta = EventMetadata.create('event', 'svc', correlationId: 'corr-127');
      expect(meta.schemaVersion, equals(1));
    });

    test('createでeventTypeとsourceが設定されること', () {
      final meta = EventMetadata.create(
        'user.created',
        'auth-service',
        correlationId: 'corr-128',
      );
      expect(meta.eventType, equals('user.created'));
      expect(meta.source, equals('auth-service'));
    });
  });

  group('EventEnvelope', () {
    test('必須フィールドを指定して生成できること', () {
      final meta = EventMetadata.create('event.v1', 'service', correlationId: 'corr-129');
      final envelope = EventEnvelope(
        topic: 'k1s0.system.user.created.v1',
        key: 'user-1',
        payload: {'id': '123'},
        metadata: meta,
      );
      expect(envelope.topic, equals('k1s0.system.user.created.v1'));
      expect(envelope.key, equals('user-1'));
      expect(envelope.payload, equals({'id': '123'}));
    });
  });

  group('NoOpEventProducer', () {
    test('publishでイベントがpublishedリストに追加されること', () async {
      final producer = NoOpEventProducer();
      final envelope = EventEnvelope(
        topic: 'test-topic',
        key: 'test-key',
        payload: 'data',
        metadata: EventMetadata.create('event', 'svc', correlationId: 'corr-130'),
      );
      await producer.publish(envelope);
      expect(producer.published, hasLength(1));
      expect(producer.published.first, equals(envelope));
    });

    test('複数イベントをpublishできること', () async {
      final producer = NoOpEventProducer();
      for (var i = 0; i < 3; i++) {
        await producer.publish(EventEnvelope(
          topic: 'topic',
          key: 'key-$i',
          payload: i,
          metadata: EventMetadata.create('event', 'svc', correlationId: 'corr-b-$i'),
        ));
      }
      expect(producer.published, hasLength(3));
    });

    test('publishBatchで複数イベントが追加されること', () async {
      final producer = NoOpEventProducer();
      await producer.publishBatch([
        EventEnvelope(
          topic: 'topic',
          key: 'k1',
          payload: 1,
          metadata: EventMetadata.create('event', 'svc', correlationId: 'corr-b1'),
        ),
        EventEnvelope(
          topic: 'topic',
          key: 'k2',
          payload: 2,
          metadata: EventMetadata.create('event', 'svc', correlationId: 'corr-b2'),
        ),
      ]);
      expect(producer.published, hasLength(2));
    });

    test('closeがエラーなく完了すること', () async {
      final producer = NoOpEventProducer();
      await expectLater(producer.close(), completes);
    });
  });

  group('MessagingError', () {
    test('toStringにop名が含まれること', () {
      final err = MessagingError('Publish');
      expect(err.toString(), contains('Publish'));
    });

    test('toStringに原因が含まれること', () {
      final cause = Exception('connection refused');
      final err = MessagingError('Publish', cause: cause);
      expect(err.toString(), contains('Publish'));
      expect(err.toString(), contains('connection refused'));
    });
  });
}
