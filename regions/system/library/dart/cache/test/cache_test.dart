import 'package:test/test.dart';

import 'package:k1s0_cache/cache.dart';

void main() {
  late InMemoryCacheClient cache;

  setUp(() {
    cache = InMemoryCacheClient();
  });

  group('get/set', () {
    test('存在しないキーに対してnullを返すこと', () async {
      expect(await cache.get('missing'), isNull);
    });

    test('値を保存して取得できること', () async {
      await cache.set('key', 'value');
      expect(await cache.get('key'), equals('value'));
    });

    test('既存の値を上書きできること', () async {
      await cache.set('key', 'v1');
      await cache.set('key', 'v2');
      expect(await cache.get('key'), equals('v2'));
    });

    test('有効期限切れのエントリに対してnullを返すこと', () async {
      await cache.set('key', 'value', ttlMs: 1);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await cache.get('key'), isNull);
    });

    test('TTL期限内は値を返すこと', () async {
      await cache.set('key', 'value', ttlMs: 60000);
      expect(await cache.get('key'), equals('value'));
    });
  });

  group('delete', () {
    test('存在しないキーに対してfalseを返すこと', () async {
      expect(await cache.delete('missing'), isFalse);
    });

    test('既存のキーを削除できること', () async {
      await cache.set('key', 'value');
      expect(await cache.delete('key'), isTrue);
      expect(await cache.get('key'), isNull);
    });
  });

  group('exists', () {
    test('存在しないキーに対してfalseを返すこと', () async {
      expect(await cache.exists('missing'), isFalse);
    });

    test('存在するキーに対してtrueを返すこと', () async {
      await cache.set('key', 'value');
      expect(await cache.exists('key'), isTrue);
    });

    test('有効期限切れのキーに対してfalseを返すこと', () async {
      await cache.set('key', 'value', ttlMs: 1);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await cache.exists('key'), isFalse);
    });
  });

  group('setNX', () {
    test('キーが存在しない場合に値をセットできること', () async {
      expect(await cache.setNX('key', 'value', 60000), isTrue);
      expect(await cache.get('key'), equals('value'));
    });

    test('キーが既に存在する場合にfalseを返すこと', () async {
      await cache.set('key', 'existing');
      expect(await cache.setNX('key', 'new', 60000), isFalse);
      expect(await cache.get('key'), equals('existing'));
    });

    test('既存キーが有効期限切れの場合に値をセットできること', () async {
      await cache.set('key', 'old', ttlMs: 1);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await cache.setNX('key', 'new', 60000), isTrue);
      expect(await cache.get('key'), equals('new'));
    });
  });

  group('CacheError', () {
    test('正しいフィールドを持つこと', () {
      const err = CacheError('test message', 'TEST_CODE');
      expect(err.message, equals('test message'));
      expect(err.code, equals('TEST_CODE'));
      expect(err.toString(), contains('TEST_CODE'));
    });
  });
}
