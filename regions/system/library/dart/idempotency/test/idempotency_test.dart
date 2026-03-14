import 'package:test/test.dart';

import 'package:k1s0_idempotency/idempotency.dart';

void main() {
  late InMemoryIdempotencyStore store;

  setUp(() {
    store = InMemoryIdempotencyStore();
  });

  group('IdempotencyRecord', () {
    test('createがデフォルト値を設定すること', () {
      final record = IdempotencyRecord.create('key-1');
      expect(record.key, equals('key-1'));
      expect(record.status, equals(IdempotencyStatus.pending));
      expect(record.expiresAt, isNull);
    });

    test('TTL指定時に有効期限が設定されること', () {
      final record = IdempotencyRecord.create('key-2', ttlSecs: 60);
      expect(record.expiresAt, isNotNull);
      expect(
        record.expiresAt!.isAfter(record.createdAt),
        isTrue,
      );
    });
  });

  group('get/insert', () {
    test('存在しないキーでnullを返すこと', () async {
      expect(await store.get('missing'), isNull);
    });

    test('レコードを挿入して取得できること', () async {
      final record = IdempotencyRecord.create('key-1');
      await store.insert(record);
      final result = await store.get('key-1');
      expect(result, isNotNull);
      expect(result!.key, equals('key-1'));
      expect(result.status, equals(IdempotencyStatus.pending));
    });

    test('重複挿入時にDuplicateKeyErrorを投げること', () async {
      await store.insert(IdempotencyRecord.create('dup'));
      expect(
        () => store.insert(IdempotencyRecord.create('dup')),
        throwsA(isA<DuplicateKeyError>()),
      );
    });
  });

  group('update', () {
    test('ステータスとレスポンスを更新すること', () async {
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

    test('存在しないキーでエラーを投げること', () async {
      expect(
        () => store.update('missing', IdempotencyStatus.failed),
        throwsA(isA<IdempotencyError>()),
      );
    });
  });

  group('delete', () {
    test('存在しないキーでfalseを返すこと', () async {
      expect(await store.delete('missing'), isFalse);
    });

    test('既存レコードを削除すること', () async {
      await store.insert(IdempotencyRecord.create('key-1'));
      expect(await store.delete('key-1'), isTrue);
      expect(await store.get('key-1'), isNull);
    });
  });

  group('expiration', () {
    test('期限切れレコードが自動削除されること', () async {
      await store.insert(IdempotencyRecord.create('exp', ttlSecs: 0));
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await store.get('exp'), isNull);
    });
  });

  group('DuplicateKeyError', () {
    test('正しいメッセージを持つこと', () {
      const err = DuplicateKeyError('test-key');
      expect(err.key, equals('test-key'));
      expect(err.toString(), contains('duplicate key'));
    });
  });

  group('IdempotencyError', () {
    test('正しいフィールドを持つこと', () {
      const err = IdempotencyError('not found', 'NOT_FOUND');
      expect(err.message, equals('not found'));
      expect(err.code, equals('NOT_FOUND'));
      expect(err.toString(), contains('NOT_FOUND'));
    });
  });
}
