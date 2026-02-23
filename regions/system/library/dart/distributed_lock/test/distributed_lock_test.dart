import 'package:test/test.dart';

import 'package:k1s0_distributed_lock/distributed_lock.dart';

void main() {
  late InMemoryDistributedLock lock;

  setUp(() {
    lock = InMemoryDistributedLock();
  });

  group('acquire', () {
    test('acquires lock successfully', () async {
      final guard = await lock.acquire('key1', const Duration(seconds: 10));
      expect(guard.key, equals('key1'));
      expect(guard.token, isNotEmpty);
    });

    test('throws when lock already held', () async {
      await lock.acquire('key1', const Duration(seconds: 10));
      expect(
        () => lock.acquire('key1', const Duration(seconds: 10)),
        throwsA(isA<LockException>()),
      );
    });

    test('allows acquire after expiry', () async {
      await lock.acquire('key1', const Duration(milliseconds: 1));
      await Future<void>.delayed(const Duration(milliseconds: 10));
      final guard = await lock.acquire('key1', const Duration(seconds: 10));
      expect(guard.key, equals('key1'));
    });
  });

  group('release', () {
    test('releases lock', () async {
      final guard = await lock.acquire('key1', const Duration(seconds: 10));
      await lock.release(guard);
      expect(await lock.isLocked('key1'), isFalse);
    });

    test('throws on token mismatch', () async {
      await lock.acquire('key1', const Duration(seconds: 10));
      final fakeGuard = LockGuard(key: 'key1', token: 'wrong-token');
      expect(
        () => lock.release(fakeGuard),
        throwsA(isA<LockException>()),
      );
    });
  });

  group('isLocked', () {
    test('returns false for unknown key', () async {
      expect(await lock.isLocked('unknown'), isFalse);
    });

    test('returns true for held lock', () async {
      await lock.acquire('key1', const Duration(seconds: 10));
      expect(await lock.isLocked('key1'), isTrue);
    });
  });
}
