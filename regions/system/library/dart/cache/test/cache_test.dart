import 'package:test/test.dart';

import 'package:k1s0_cache/cache.dart';

void main() {
  late InMemoryCacheClient cache;

  setUp(() {
    cache = InMemoryCacheClient();
  });

  group('get/set', () {
    test('returns null for missing key', () async {
      expect(await cache.get('missing'), isNull);
    });

    test('stores and retrieves value', () async {
      await cache.set('key', 'value');
      expect(await cache.get('key'), equals('value'));
    });

    test('overwrites existing value', () async {
      await cache.set('key', 'v1');
      await cache.set('key', 'v2');
      expect(await cache.get('key'), equals('v2'));
    });

    test('returns null for expired entry', () async {
      await cache.set('key', 'value', ttlMs: 1);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await cache.get('key'), isNull);
    });

    test('returns value before TTL expires', () async {
      await cache.set('key', 'value', ttlMs: 60000);
      expect(await cache.get('key'), equals('value'));
    });
  });

  group('delete', () {
    test('returns false for missing key', () async {
      expect(await cache.delete('missing'), isFalse);
    });

    test('removes existing key', () async {
      await cache.set('key', 'value');
      expect(await cache.delete('key'), isTrue);
      expect(await cache.get('key'), isNull);
    });
  });

  group('exists', () {
    test('returns false for missing key', () async {
      expect(await cache.exists('missing'), isFalse);
    });

    test('returns true for existing key', () async {
      await cache.set('key', 'value');
      expect(await cache.exists('key'), isTrue);
    });

    test('returns false for expired key', () async {
      await cache.set('key', 'value', ttlMs: 1);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await cache.exists('key'), isFalse);
    });
  });

  group('setNX', () {
    test('sets value when key does not exist', () async {
      expect(await cache.setNX('key', 'value', 60000), isTrue);
      expect(await cache.get('key'), equals('value'));
    });

    test('returns false when key exists', () async {
      await cache.set('key', 'existing');
      expect(await cache.setNX('key', 'new', 60000), isFalse);
      expect(await cache.get('key'), equals('existing'));
    });

    test('sets value when existing key is expired', () async {
      await cache.set('key', 'old', ttlMs: 1);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(await cache.setNX('key', 'new', 60000), isTrue);
      expect(await cache.get('key'), equals('new'));
    });
  });

  group('CacheError', () {
    test('has correct fields', () {
      const err = CacheError('test message', 'TEST_CODE');
      expect(err.message, equals('test message'));
      expect(err.code, equals('TEST_CODE'));
      expect(err.toString(), contains('TEST_CODE'));
    });
  });
}
