import 'package:test/test.dart';

import 'package:k1s0_idempotency/idempotency.dart';

void main() {
  late InMemoryIdempotencyStore store;

  setUp(() {
    store = InMemoryIdempotencyStore();
  });

  group('IdempotencyRecord', () {
    test('create sets defaults', () {
      final record = IdempotencyRecord.create('key-1');
      expect(record.key, equals('key-1'));
      expect(record.status, equals(IdempotencyStatus.pending));
      expect(record.expiresAt, isNull);
    });

    test('create with ttl sets expiration', () {
      final record = IdempotencyRecord.create('key-2', ttlSecs: 60);
      expect(record.expiresAt, isNotNull);
      expect(
        record.expiresAt!.isAfter(record.createdAt),
        isTrue,
      );
    });
  });

  group('get/insert', () {
    test('returns null for missing key', () async {
      expect(await store.get('missing'), isNull);
    });

    test('inserts and retrieves record', () async {
      final record = IdempotencyRecord.create('key-1');
      await store.insert(record);
      final result = await store.get('key-1');
      expect(result, isNotNull);
      expect(result!.key, equals('key-1'));
      expect(result.status, equals(IdempotencyStatus.pending));
    });

    test('throws DuplicateKeyError on duplicate insert', () async {
      await store.insert(IdempotencyRecord.create('dup'));
      expect(
        () => store.insert(IdempotencyRecord.create('dup')),
        throwsA(isA<DuplicateKeyError>()),
      );
    });
  });

  group('update', () {
    test('updates status and response', () async {
      await store.insert(IdempotencyRecord.create('key-1'));
      await store.update(
        'key-1',
        IdempotencyStatus.completed,
        body: '{"ok":true}',
        code: 200,
      );
      final record = await store.get('key-1');
      expect(record!.status, equals(IdempotencyStatus.completed));
      expect(record.responseBody, equals('{"ok":true}'));
      expect(record.statusCode, equals(200));
      expect(record.completedAt, isNotNull);
    });

    test('throws on missing key', () async {
      expect(
        () => store.update('missing', IdempotencyStatus.failed),
        throwsA(isA<IdempotencyError>()),
      );
    });
  });

  group('delete', () {
    test('returns false for missing key', () async {
      expect(await store.delete('missing'), isFalse);
    });

    test('removes existing record', () async {
      await store.insert(IdempotencyRecord.create('key-1'));
      expect(await store.delete('key-1'), isTrue);
      expect(await store.get('key-1'), isNull);
    });
  });

  group('expiration', () {
    test('expired records are cleaned up', () async {
      await store.insert(IdempotencyRecord.create('exp', ttlSecs: 0));
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await store.get('exp'), isNull);
    });
  });

  group('DuplicateKeyError', () {
    test('has correct message', () {
      const err = DuplicateKeyError('test-key');
      expect(err.key, equals('test-key'));
      expect(err.toString(), contains('duplicate key'));
    });
  });

  group('IdempotencyError', () {
    test('has correct fields', () {
      const err = IdempotencyError('not found', 'NOT_FOUND');
      expect(err.message, equals('not found'));
      expect(err.code, equals('NOT_FOUND'));
      expect(err.toString(), contains('NOT_FOUND'));
    });
  });
}
