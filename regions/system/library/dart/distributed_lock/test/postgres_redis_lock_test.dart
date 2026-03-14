import 'package:test/test.dart';
import 'package:k1s0_distributed_lock/distributed_lock.dart';

// Integration tests for PostgresDistributedLock and RedisDistributedLock
// require live infrastructure. These tests verify the contract via InMemory.

void main() {
  group('DistributedLockコントラクト (InMemory)', () {
    late InMemoryDistributedLock lock;

    setUp(() {
      lock = InMemoryDistributedLock();
    });

    test('acquireが正しいキーを持つLockGuardを返すこと', () async {
      final guard = await lock.acquire('test-key', const Duration(seconds: 30));
      expect(guard.key, equals('test-key'));
      expect(guard.token, isNotEmpty);
    });

    test('ロックが保持されている場合にacquireがLockExceptionをスローすること', () async {
      await lock.acquire('test-key', const Duration(seconds: 30));
      expect(
        () => lock.acquire('test-key', const Duration(seconds: 30)),
        throwsA(isA<LockException>()),
      );
    });

    test('releaseの後に再取得できること', () async {
      final guard = await lock.acquire('test-key', const Duration(seconds: 30));
      await lock.release(guard);
      final guard2 = await lock.acquire('test-key', const Duration(seconds: 30));
      expect(guard2.key, equals('test-key'));
    });

    test('不正なトークンでreleaseするとLockExceptionをスローすること', () async {
      await lock.acquire('test-key', const Duration(seconds: 30));
      final fakeGuard = LockGuard(key: 'test-key', token: 'invalid-token');
      expect(
        () => lock.release(fakeGuard),
        throwsA(isA<LockException>()),
      );
    });

    test('isLockedがロックされていないキーに対してfalseを返すこと', () async {
      expect(await lock.isLocked('no-key'), isFalse);
    });

    test('isLockedがロック保持中のキーに対してtrueを返すこと', () async {
      await lock.acquire('test-key', const Duration(seconds: 30));
      expect(await lock.isLocked('test-key'), isTrue);
    });

    test('isLockedがrelease後にfalseを返すこと', () async {
      final guard = await lock.acquire('test-key', const Duration(seconds: 30));
      await lock.release(guard);
      expect(await lock.isLocked('test-key'), isFalse);
    });

    test('複数のキーが互いに独立していること', () async {
      final guard1 = await lock.acquire('key1', const Duration(seconds: 30));
      final guard2 = await lock.acquire('key2', const Duration(seconds: 30));
      expect(guard1.key, equals('key1'));
      expect(guard2.key, equals('key2'));
      await lock.release(guard1);
      expect(await lock.isLocked('key2'), isTrue);
    });
  });
}
