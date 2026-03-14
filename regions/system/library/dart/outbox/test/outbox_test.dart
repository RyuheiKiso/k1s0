import 'package:test/test.dart';
import 'package:k1s0_outbox/outbox.dart';

class MockStore implements OutboxStore {
  List<OutboxMessage> messages = [];
  List<OutboxMessage> savedMessages = [];
  List<OutboxMessage> updatedMessages = [];
  int deletedCount = 0;
  Exception? fetchError;
  Exception? updateError;

  @override
  Future<void> save(OutboxMessage msg) async {
    savedMessages.add(msg);
  }

  @override
  Future<List<OutboxMessage>> fetchPending(int limit) async {
    if (fetchError != null) throw fetchError!;
    return messages.take(limit).toList();
  }

  @override
  Future<void> update(OutboxMessage msg) async {
    if (updateError != null) throw updateError!;
    // Store a snapshot of current state
    updatedMessages.add(OutboxMessage(
      id: msg.id,
      topic: msg.topic,
      partitionKey: msg.partitionKey,
      payload: msg.payload,
      status: msg.status,
      retryCount: msg.retryCount,
      maxRetries: msg.maxRetries,
      lastError: msg.lastError,
      createdAt: msg.createdAt,
      processAfter: msg.processAfter,
    ));
  }

  @override
  Future<int> deleteDelivered(int olderThanDays) async {
    return deletedCount;
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
  String partitionKey = 'test-key',
  String payload = '{"key":"value"}',
  OutboxStatus status = OutboxStatus.pending,
  int retryCount = 0,
  int maxRetries = 3,
}) {
  final now = DateTime.now().toUtc();
  return OutboxMessage(
    id: id,
    topic: topic,
    partitionKey: partitionKey,
    payload: payload,
    status: status,
    retryCount: retryCount,
    maxRetries: maxRetries,
    lastError: null,
    createdAt: now,
    processAfter: now,
  );
}

void main() {
  group('createOutboxMessage', () {
    test('正しいフィールドを持つメッセージが生成されること', () {
      final msg = createOutboxMessage('test-topic', 'test-key', '{"id":"1"}');
      expect(msg.topic, equals('test-topic'));
      expect(msg.partitionKey, equals('test-key'));
      expect(msg.payload, equals('{"id":"1"}'));
      expect(msg.status, equals(OutboxStatus.pending));
      expect(msg.retryCount, equals(0));
      expect(msg.maxRetries, equals(3));
      expect(msg.lastError, isNull);
      expect(msg.createdAt.isUtc, isTrue);
      expect(msg.processAfter.isUtc, isTrue);
    });

    test('一意なIDが生成されること', () {
      final msg1 = createOutboxMessage('t', 'k', 'p');
      final msg2 = createOutboxMessage('t', 'k', 'p');
      expect(msg1.id, isNot(equals(msg2.id)));
    });
  });

  group('markProcessing', () {
    test('ステータスがprocessingに設定されること', () {
      final msg = createOutboxMessage('topic', 'key', '{}');
      msg.markProcessing();
      expect(msg.status, equals(OutboxStatus.processing));
    });
  });

  group('markDelivered', () {
    test('ステータスがdeliveredに設定されること', () {
      final msg = createOutboxMessage('topic', 'key', '{}');
      msg.markProcessing();
      msg.markDelivered();
      expect(msg.status, equals(OutboxStatus.delivered));
      expect(msg.isProcessable, isFalse);
    });
  });

  group('markFailed', () {
    test('retryCountが増加しステータスがfailedに設定されること', () {
      final msg = createOutboxMessage('topic', 'key', '{}');
      msg.markFailed('kafka error');
      expect(msg.retryCount, equals(1));
      expect(msg.status, equals(OutboxStatus.failed));
      expect(msg.lastError, equals('kafka error'));
    });

    test('最大リトライ数に達するとdeadLetterに設定されること', () {
      final msg = createOutboxMessage('topic', 'key', '{}');
      msg.maxRetries = 3;
      msg.markFailed('error 1');
      msg.markFailed('error 2');
      msg.markFailed('error 3');
      expect(msg.status, equals(OutboxStatus.deadLetter));
      expect(msg.retryCount, equals(3));
    });

    test('指数バックオフが秒単位で適用されること', () {
      final msg = createOutboxMessage('topic', 'key', '{}');
      final before = DateTime.now().toUtc();
      msg.markFailed('error');
      // retryCount is now 1, so delay = 2^1 = 2 seconds
      final expectedMin = before.add(const Duration(seconds: 2));
      expect(
        msg.processAfter.isAfter(expectedMin) ||
            msg.processAfter.isAtSameMomentAs(expectedMin) ||
            msg.processAfter
                .difference(expectedMin)
                .inMilliseconds
                .abs() < 100,
        isTrue,
      );
    });
  });

  group('isProcessable', () {
    test('processAfterが過去のpendingメッセージはtrueを返すこと', () {
      final msg = createOutboxMessage('topic', 'key', '{}');
      expect(msg.isProcessable, isTrue);
    });

    test('processAfterが過去のfailedメッセージはtrueを返すこと', () {
      final msg = _createTestMessage(status: OutboxStatus.failed);
      msg.processAfter = DateTime.now().toUtc().subtract(const Duration(seconds: 1));
      expect(msg.isProcessable, isTrue);
    });

    test('deliveredメッセージはfalseを返すこと', () {
      final msg = _createTestMessage(status: OutboxStatus.delivered);
      expect(msg.isProcessable, isFalse);
    });

    test('deadLetterメッセージはfalseを返すこと', () {
      final msg = _createTestMessage(status: OutboxStatus.deadLetter);
      expect(msg.isProcessable, isFalse);
    });

    test('processAfterが未来の場合はfalseを返すこと', () {
      final msg = createOutboxMessage('topic', 'key', '{}');
      msg.processAfter = DateTime.now().toUtc().add(const Duration(minutes: 1));
      expect(msg.isProcessable, isFalse);
    });
  });

  group('canTransitionTo', () {
    test('pending -> processingの遷移が有効であること', () {
      expect(
          canTransitionTo(OutboxStatus.pending, OutboxStatus.processing),
          isTrue);
    });

    test('pending -> deliveredの遷移が無効であること', () {
      expect(
          canTransitionTo(OutboxStatus.pending, OutboxStatus.delivered),
          isFalse);
    });

    test('pending -> failedの遷移が無効であること', () {
      expect(canTransitionTo(OutboxStatus.pending, OutboxStatus.failed),
          isFalse);
    });

    test('processing -> deliveredの遷移が有効であること', () {
      expect(
          canTransitionTo(OutboxStatus.processing, OutboxStatus.delivered),
          isTrue);
    });

    test('processing -> failedの遷移が有効であること', () {
      expect(
          canTransitionTo(OutboxStatus.processing, OutboxStatus.failed),
          isTrue);
    });

    test('processing -> deadLetterの遷移が有効であること', () {
      expect(
          canTransitionTo(OutboxStatus.processing, OutboxStatus.deadLetter),
          isTrue);
    });

    test('processing -> pendingの遷移が無効であること', () {
      expect(
          canTransitionTo(OutboxStatus.processing, OutboxStatus.pending),
          isFalse);
    });

    test('failed -> processingの遷移が有効であること', () {
      expect(
          canTransitionTo(OutboxStatus.failed, OutboxStatus.processing),
          isTrue);
    });

    test('failed -> deliveredの遷移が無効であること', () {
      expect(
          canTransitionTo(OutboxStatus.failed, OutboxStatus.delivered),
          isFalse);
    });

    test('delivered -> pendingの遷移が無効であること', () {
      expect(
          canTransitionTo(OutboxStatus.delivered, OutboxStatus.pending),
          isFalse);
    });

    test('delivered -> processingの遷移が無効であること', () {
      expect(
          canTransitionTo(OutboxStatus.delivered, OutboxStatus.processing),
          isFalse);
    });

    test('delivered -> failedの遷移が無効であること', () {
      expect(
          canTransitionTo(OutboxStatus.delivered, OutboxStatus.failed),
          isFalse);
    });

    test('deadLetter -> pendingの遷移が無効であること', () {
      expect(
          canTransitionTo(OutboxStatus.deadLetter, OutboxStatus.pending),
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

    test('processBatchで成功メッセージが処理されること', () async {
      final msg1 = _createTestMessage(id: 'msg-1');
      final msg2 = _createTestMessage(id: 'msg-2');
      store.messages = [msg1, msg2];

      final processor = OutboxProcessor(store, publisher);
      final count = await processor.processBatch();

      expect(count, equals(2));
      expect(publisher.published, hasLength(2));

      // Verify status updates: processing, delivered for each message
      expect(store.updatedMessages, hasLength(4));
      expect(store.updatedMessages[0].id, equals('msg-1'));
      expect(store.updatedMessages[0].status, equals(OutboxStatus.processing));
      expect(store.updatedMessages[1].id, equals('msg-1'));
      expect(store.updatedMessages[1].status, equals(OutboxStatus.delivered));
      expect(store.updatedMessages[2].id, equals('msg-2'));
      expect(store.updatedMessages[2].status, equals(OutboxStatus.processing));
      expect(store.updatedMessages[3].id, equals('msg-2'));
      expect(store.updatedMessages[3].status, equals(OutboxStatus.delivered));
    });

    test('processBatchでpublishエラー時にfailedとマークされること', () async {
      final msg = _createTestMessage(id: 'msg-fail');
      store.messages = [msg];
      publisher.error = Exception('publish failed');

      final processor = OutboxProcessor(store, publisher);
      final count = await processor.processBatch();

      expect(count, equals(0));
      // processing, then failed
      expect(store.updatedMessages, hasLength(2));
      expect(store.updatedMessages[0].status, equals(OutboxStatus.processing));
      expect(store.updatedMessages[1].status, equals(OutboxStatus.failed));
      expect(store.updatedMessages[1].retryCount, equals(1));
      expect(store.updatedMessages[1].lastError, isNotNull);
    });

    test('processBatchで最大リトライ後にdeadLetterとマークされること', () async {
      final msg = _createTestMessage(id: 'msg-dead', maxRetries: 1);
      store.messages = [msg];
      publisher.error = Exception('always fail');

      final processor = OutboxProcessor(store, publisher);
      final count = await processor.processBatch();

      expect(count, equals(0));
      expect(store.updatedMessages, hasLength(2));
      expect(store.updatedMessages[1].status, equals(OutboxStatus.deadLetter));
      expect(store.updatedMessages[1].retryCount, equals(1));
    });

    test('processBatchでfetchPendingエラー時に例外がスローされること', () async {
      store.fetchError = Exception('db connection failed');

      final processor = OutboxProcessor(store, publisher);
      expect(
        () => processor.processBatch(),
        throwsA(isA<Exception>()),
      );
    });

    test('processBatchで空バッチの場合0が返されること', () async {
      store.messages = [];

      final processor = OutboxProcessor(store, publisher);
      final count = await processor.processBatch();

      expect(count, equals(0));
      expect(publisher.published, isEmpty);
    });

    test('processBatchでbatchSizeが考慮されること', () async {
      store.messages = List.generate(
        10,
        (i) => _createTestMessage(id: 'msg-$i'),
      );

      final processor = OutboxProcessor(store, publisher, batchSize: 3);
      final count = await processor.processBatch();

      expect(count, equals(3));
      expect(publisher.published, hasLength(3));
    });

    test('0または負の値の場合batchSizeのデフォルトが100になること', () async {
      store.messages = [];

      final processor1 = OutboxProcessor(store, publisher, batchSize: 0);
      expect(processor1.batchSize, equals(100));

      final processor2 = OutboxProcessor(store, publisher, batchSize: -5);
      expect(processor2.batchSize, equals(100));
    });
  });

  group('OutboxStore interface', () {
    test('saveでメッセージが保存されること', () async {
      final store = MockStore();
      final msg = createOutboxMessage('topic', 'key', '{}');
      await store.save(msg);
      expect(store.savedMessages, hasLength(1));
    });

    test('deleteDeliveredで削除件数が返されること', () async {
      final store = MockStore();
      store.deletedCount = 5;
      final count = await store.deleteDelivered(30);
      expect(count, equals(5));
    });
  });

  group('OutboxError', () {
    test('toStringにエラーコードが含まれること', () {
      const err = OutboxError(OutboxErrorCode.storeError);
      expect(err.toString(), contains('storeError'));
    });

    test('toStringにメッセージが含まれること', () {
      const err =
          OutboxError(OutboxErrorCode.publishError, message: 'kafka down');
      expect(err.toString(), contains('publishError'));
      expect(err.toString(), contains('kafka down'));
    });

    test('toStringに原因が含まれること', () {
      final cause = Exception('db error');
      final err = OutboxError(OutboxErrorCode.storeError,
          message: 'connection failed', cause: cause);
      expect(err.toString(), contains('storeError'));
      expect(err.toString(), contains('connection failed'));
      expect(err.toString(), contains('db error'));
    });

    test('全エラーコードがサポートされること', () {
      for (final code in OutboxErrorCode.values) {
        final err = OutboxError(code, message: 'test');
        expect(err.code, equals(code));
      }
    });
  });
}
