import 'package:test/test.dart';
import 'package:k1s0_outbox/outbox.dart';

class StatusUpdate {
  final String id;
  final OutboxStatus status;
  final int? retryCount;
  final DateTime? scheduledAt;

  StatusUpdate(this.id, this.status, {this.retryCount, this.scheduledAt});
}

class MockStore implements OutboxStore {
  List<OutboxMessage> messages = [];
  List<OutboxMessage> savedMessages = [];
  List<StatusUpdate> statusUpdates = [];
  Exception? getError;
  Exception? updateError;

  @override
  Future<void> saveMessage(OutboxMessage msg) async {
    savedMessages.add(msg);
  }

  @override
  Future<List<OutboxMessage>> getPendingMessages(int limit) async {
    if (getError != null) throw getError!;
    return messages.take(limit).toList();
  }

  @override
  Future<void> updateStatus(String id, OutboxStatus status) async {
    if (updateError != null) throw updateError!;
    statusUpdates.add(StatusUpdate(id, status));
  }

  @override
  Future<void> updateStatusWithRetry(
    String id,
    OutboxStatus status,
    int retryCount,
    DateTime scheduledAt,
  ) async {
    statusUpdates.add(
      StatusUpdate(id, status,
          retryCount: retryCount, scheduledAt: scheduledAt),
    );
  }
}

class MockPublisher implements OutboxPublisher {
  List<OutboxMessage> published = [];
  Exception? error;

  @override
  Future<void> publish(OutboxMessage msg) async {
    if (error != null) throw error!;
    published.add(msg);
  }
}

OutboxMessage _createTestMessage({
  String id = 'test-id',
  String topic = 'test-topic',
  String eventType = 'test.event',
  String payload = '{"key":"value"}',
  OutboxStatus status = OutboxStatus.pending,
  int retryCount = 0,
  String correlationId = 'corr-123',
}) {
  final now = DateTime.now().toUtc();
  return OutboxMessage(
    id: id,
    topic: topic,
    eventType: eventType,
    payload: payload,
    status: status,
    retryCount: retryCount,
    scheduledAt: now,
    createdAt: now,
    updatedAt: now,
    correlationId: correlationId,
  );
}

void main() {
  group('createOutboxMessage', () {
    test('creates message with correct fields', () {
      final msg = createOutboxMessage(
          'test-topic', 'user.created', '{"id":"1"}', 'corr-123');
      expect(msg.topic, equals('test-topic'));
      expect(msg.eventType, equals('user.created'));
      expect(msg.payload, equals('{"id":"1"}'));
      expect(msg.correlationId, equals('corr-123'));
      expect(msg.status, equals(OutboxStatus.pending));
      expect(msg.retryCount, equals(0));
      expect(msg.createdAt.isUtc, isTrue);
      expect(msg.updatedAt.isUtc, isTrue);
      expect(msg.scheduledAt.isUtc, isTrue);
    });

    test('generates unique IDs', () {
      final msg1 = createOutboxMessage('t', 'e', 'p', 'c');
      final msg2 = createOutboxMessage('t', 'e', 'p', 'c');
      expect(msg1.id, isNot(equals(msg2.id)));
    });
  });

  group('nextScheduledAt', () {
    test('retryCount 0 returns ~1 minute delay', () {
      final before = DateTime.now().toUtc();
      final scheduled = nextScheduledAt(0);
      final expectedMin = before.add(const Duration(minutes: 1));
      expect(scheduled.isAfter(expectedMin) || scheduled == expectedMin,
          isTrue);
    });

    test('retryCount 1 returns ~2 minutes delay', () {
      final before = DateTime.now().toUtc();
      final scheduled = nextScheduledAt(1);
      final expectedMin = before.add(const Duration(minutes: 2));
      expect(scheduled.isAfter(expectedMin) || scheduled == expectedMin,
          isTrue);
    });

    test('retryCount 2 returns ~4 minutes delay', () {
      final before = DateTime.now().toUtc();
      final scheduled = nextScheduledAt(2);
      final expectedMin = before.add(const Duration(minutes: 4));
      expect(scheduled.isAfter(expectedMin) || scheduled == expectedMin,
          isTrue);
    });

    test('retryCount 3 returns ~8 minutes delay', () {
      final before = DateTime.now().toUtc();
      final scheduled = nextScheduledAt(3);
      final expectedMin = before.add(const Duration(minutes: 8));
      expect(scheduled.isAfter(expectedMin) || scheduled == expectedMin,
          isTrue);
    });

    test('retryCount 6 caps at 60 minutes', () {
      final before = DateTime.now().toUtc();
      final scheduled = nextScheduledAt(6);
      final expectedMin = before.add(const Duration(minutes: 60));
      // 2^6 = 64, but capped at 60
      expect(scheduled.isAfter(expectedMin) || scheduled == expectedMin,
          isTrue);
      final expectedMax =
          before.add(const Duration(minutes: 61));
      expect(scheduled.isBefore(expectedMax), isTrue);
    });

    test('retryCount 7 caps at 60 minutes', () {
      final before = DateTime.now().toUtc();
      final scheduled = nextScheduledAt(7);
      final expectedMin = before.add(const Duration(minutes: 60));
      expect(scheduled.isAfter(expectedMin) || scheduled == expectedMin,
          isTrue);
      final expectedMax =
          before.add(const Duration(minutes: 61));
      expect(scheduled.isBefore(expectedMax), isTrue);
    });
  });

  group('canTransitionTo', () {
    test('pending -> processing is valid', () {
      expect(
          canTransitionTo(OutboxStatus.pending, OutboxStatus.processing),
          isTrue);
    });

    test('pending -> delivered is invalid', () {
      expect(
          canTransitionTo(OutboxStatus.pending, OutboxStatus.delivered),
          isFalse);
    });

    test('pending -> failed is invalid', () {
      expect(canTransitionTo(OutboxStatus.pending, OutboxStatus.failed),
          isFalse);
    });

    test('processing -> delivered is valid', () {
      expect(
          canTransitionTo(OutboxStatus.processing, OutboxStatus.delivered),
          isTrue);
    });

    test('processing -> failed is valid', () {
      expect(
          canTransitionTo(OutboxStatus.processing, OutboxStatus.failed),
          isTrue);
    });

    test('processing -> pending is invalid', () {
      expect(
          canTransitionTo(OutboxStatus.processing, OutboxStatus.pending),
          isFalse);
    });

    test('failed -> pending is valid', () {
      expect(canTransitionTo(OutboxStatus.failed, OutboxStatus.pending),
          isTrue);
    });

    test('failed -> processing is invalid', () {
      expect(
          canTransitionTo(OutboxStatus.failed, OutboxStatus.processing),
          isFalse);
    });

    test('failed -> delivered is invalid', () {
      expect(
          canTransitionTo(OutboxStatus.failed, OutboxStatus.delivered),
          isFalse);
    });

    test('delivered -> pending is invalid', () {
      expect(
          canTransitionTo(OutboxStatus.delivered, OutboxStatus.pending),
          isFalse);
    });

    test('delivered -> processing is invalid', () {
      expect(
          canTransitionTo(OutboxStatus.delivered, OutboxStatus.processing),
          isFalse);
    });

    test('delivered -> failed is invalid', () {
      expect(
          canTransitionTo(OutboxStatus.delivered, OutboxStatus.failed),
          isFalse);
    });
  });

  group('OutboxProcessor', () {
    late MockStore store;
    late MockPublisher publisher;

    setUp(() {
      store = MockStore();
      publisher = MockPublisher();
    });

    test('processBatch processes successful messages', () async {
      final msg1 = _createTestMessage(id: 'msg-1');
      final msg2 = _createTestMessage(id: 'msg-2');
      store.messages = [msg1, msg2];

      final processor = OutboxProcessor(store, publisher);
      final count = await processor.processBatch();

      expect(count, equals(2));
      expect(publisher.published, hasLength(2));

      // Verify status updates: processing, delivered for each message
      expect(store.statusUpdates, hasLength(4));
      expect(store.statusUpdates[0].id, equals('msg-1'));
      expect(store.statusUpdates[0].status, equals(OutboxStatus.processing));
      expect(store.statusUpdates[1].id, equals('msg-1'));
      expect(store.statusUpdates[1].status, equals(OutboxStatus.delivered));
      expect(store.statusUpdates[2].id, equals('msg-2'));
      expect(store.statusUpdates[2].status, equals(OutboxStatus.processing));
      expect(store.statusUpdates[3].id, equals('msg-2'));
      expect(store.statusUpdates[3].status, equals(OutboxStatus.delivered));
    });

    test('processBatch marks failed on publish error', () async {
      final msg = _createTestMessage(id: 'msg-fail');
      store.messages = [msg];
      publisher.error = Exception('publish failed');

      final processor = OutboxProcessor(store, publisher);
      final count = await processor.processBatch();

      expect(count, equals(0));
      // processing, then failed via updateStatusWithRetry
      expect(store.statusUpdates, hasLength(2));
      expect(store.statusUpdates[0].status, equals(OutboxStatus.processing));
      expect(store.statusUpdates[1].status, equals(OutboxStatus.failed));
      expect(store.statusUpdates[1].retryCount, equals(1));
    });

    test('processBatch throws on store getPendingMessages error', () async {
      store.getError = Exception('db connection failed');

      final processor = OutboxProcessor(store, publisher);
      expect(
        () => processor.processBatch(),
        throwsA(isA<OutboxError>()
            .having((e) => e.op, 'op', equals('GetPendingMessages'))),
      );
    });

    test('processBatch returns 0 for empty batch', () async {
      store.messages = [];

      final processor = OutboxProcessor(store, publisher);
      final count = await processor.processBatch();

      expect(count, equals(0));
      expect(publisher.published, isEmpty);
    });

    test('processBatch respects batchSize', () async {
      store.messages = List.generate(
        10,
        (i) => _createTestMessage(id: 'msg-$i'),
      );

      final processor = OutboxProcessor(store, publisher, batchSize: 3);
      final count = await processor.processBatch();

      expect(count, equals(3));
      expect(publisher.published, hasLength(3));
    });

    test('default batchSize is 100 when 0 or negative', () async {
      store.messages = [];

      final processor1 = OutboxProcessor(store, publisher, batchSize: 0);
      expect(processor1.batchSize, equals(100));

      final processor2 = OutboxProcessor(store, publisher, batchSize: -5);
      expect(processor2.batchSize, equals(100));
    });
  });

  group('OutboxError', () {
    test('toString includes op', () {
      const err = OutboxError('ProcessBatch');
      expect(err.toString(), contains('ProcessBatch'));
    });

    test('toString includes cause', () {
      final cause = Exception('db error');
      final err = OutboxError('ProcessBatch', cause: cause);
      expect(err.toString(), contains('ProcessBatch'));
      expect(err.toString(), contains('db error'));
    });
  });
}
