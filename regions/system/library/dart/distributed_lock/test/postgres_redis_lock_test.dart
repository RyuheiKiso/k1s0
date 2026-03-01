import 'package:test/test.dart';
import 'package:k1s0_distributed_lock/distributed_lock.dart';

// Integration tests for PostgresDistributedLock and RedisDistributedLock
// require live infrastructure. These tests verify the contract via InMemory.

void main() {
  group('DistributedLock contract (InMemory)', () {
    late InMemoryDistributedLock lock;

    setUp(() {
      lock = InMemoryDistributedLock();
    });

    test('acquire returns LockGuard with correct key', () async {
      final guard = await lock.acquire('test-key', const Duration(seconds: 30));
      expect(guard.key, equals('test-key'));
      expect(guard.token, isNotEmpty);
    });

    test('acquire throws LockException when lock is held', () async {
      await lock.acquire('test-key', const Duration(seconds: 30));
      expect(
        () => lock.acquire('test-key', const Duration(seconds: 30)),
        throwsA(isA<LockException>()),
      );
    });

    test('release allows re-acquire', () async {
      final guard = await lock.acquire('test-key', const Duration(seconds: 30));
      await lock.release(guard);
      final guard2 = await lock.acquire('test-key', const Duration(seconds: 30));
      expect(guard2.key, equals('test-key'));
    });

    test('release with wrong token throws LockException', () async {
      await lock.acquire('test-key', const Duration(seconds: 30));
      final fakeGuard = LockGuard(key: 'test-key', token: 'invalid-token');
      expect(
        () => lock.release(fakeGuard),
        throwsA(isA<LockException>()),
      );
    });

    test('isLocked returns false for unheld key', () async {
      expect(await lock.isLocked('no-key'), isFalse);
    });

    test('isLocked returns true for held lock', () async {
      await lock.acquire('test-key', const Duration(seconds: 30));
      expect(await lock.isLocked('test-key'), isTrue);
    });

    test('isLocked returns false after release', () async {
      final guard = await lock.acquire('test-key', const Duration(seconds: 30));
      await lock.release(guard);
      expect(await lock.isLocked('test-key'), isFalse);
    });

    test('multiple keys are independent', () async {
      final guard1 = await lock.acquire('key1', const Duration(seconds: 30));
      final guard2 = await lock.acquire('key2', const Duration(seconds: 30));
      expect(guard1.key, equals('key1'));
      expect(guard2.key, equals('key2'));
      await lock.release(guard1);
      expect(await lock.isLocked('key2'), isTrue);
    });
  });
}
