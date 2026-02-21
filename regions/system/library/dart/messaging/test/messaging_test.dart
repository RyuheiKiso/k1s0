import 'package:test/test.dart';
import 'package:k1s0_messaging/messaging.dart';

void main() {
  group('EventMetadata', () {
    test('create generates UUID eventId', () {
      final meta = EventMetadata.create('user.created', 'auth-service');
      expect(meta.eventId, isNotEmpty);
      final uuidRegex = RegExp(
          r'^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$');
      expect(uuidRegex.hasMatch(meta.eventId), isTrue);
    });

    test('create uses provided correlationId', () {
      final meta =
          EventMetadata.create('event', 'svc', correlationId: 'corr-123');
      expect(meta.correlationId, equals('corr-123'));
    });

    test('create uses provided traceId', () {
      final meta =
          EventMetadata.create('event', 'svc', traceId: 'trace-123');
      expect(meta.traceId, equals('trace-123'));
    });

    test('create auto-generates correlationId and traceId', () {
      final meta = EventMetadata.create('event', 'svc');
      expect(meta.correlationId, isNotEmpty);
      expect(meta.traceId, isNotEmpty);
    });

    test('create sets timestamp to UTC', () {
      final meta = EventMetadata.create('event', 'svc');
      expect(meta.timestamp.isUtc, isTrue);
    });

    test('create sets eventType and source', () {
      final meta = EventMetadata.create('user.created', 'auth-service');
      expect(meta.eventType, equals('user.created'));
      expect(meta.source, equals('auth-service'));
    });
  });

  group('EventEnvelope', () {
    test('can be created with required fields', () {
      final meta = EventMetadata.create('event.v1', 'service');
      final envelope = EventEnvelope(
        topic: 'k1s0.system.user.created.v1',
        payload: {'id': '123'},
        metadata: meta,
      );
      expect(envelope.topic, equals('k1s0.system.user.created.v1'));
      expect(envelope.payload, equals({'id': '123'}));
    });
  });

  group('NoOpEventProducer', () {
    test('publish adds event to published list', () async {
      final producer = NoOpEventProducer();
      final envelope = EventEnvelope(
        topic: 'test-topic',
        payload: 'data',
        metadata: EventMetadata.create('event', 'svc'),
      );
      await producer.publish(envelope);
      expect(producer.published, hasLength(1));
      expect(producer.published.first, equals(envelope));
    });

    test('publish multiple events', () async {
      final producer = NoOpEventProducer();
      for (var i = 0; i < 3; i++) {
        await producer.publish(EventEnvelope(
          topic: 'topic',
          payload: i,
          metadata: EventMetadata.create('event', 'svc'),
        ));
      }
      expect(producer.published, hasLength(3));
    });

    test('close completes without error', () async {
      final producer = NoOpEventProducer();
      await expectLater(producer.close(), completes);
    });
  });

  group('MessagingError', () {
    test('toString includes op', () {
      final err = MessagingError('Publish');
      expect(err.toString(), contains('Publish'));
    });

    test('toString includes cause', () {
      final cause = Exception('connection refused');
      final err = MessagingError('Publish', cause: cause);
      expect(err.toString(), contains('Publish'));
      expect(err.toString(), contains('connection refused'));
    });
  });
}
