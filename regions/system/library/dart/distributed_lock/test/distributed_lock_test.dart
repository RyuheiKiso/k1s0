import 'package:test/test.dart';

import 'package:k1s0_distributed_lock/distributed_lock.dart';

void main() {
  late InMemoryDistributedLock lock;

  setUp(() {
    lock = InMemoryDistributedLock();
  });

  group('acquire', () {
    test('ロックを正常に取得できること', () async {
      final guard = await lock.acquire('key1', const Duration(seconds: 10));
      expect(guard.key, equals('key1'));
      expect(guard.token, isNotEmpty);
    });

    test('ロックが既に保持されている場合に例外をスローすること', () async {
      await lock.acquire('key1', const Duration(seconds: 10));
      expect(
        () => lock.acquire('key1', const Duration(seconds: 10)),
        throwsA(isA<LockException>()),
      );
    });

    test('有効期限切れ後に再取得できること', () async {
      await lock.acquire('key1', const Duration(milliseconds: 1));
      await Future<void>.delayed(const Duration(milliseconds: 10));
      final guard = await lock.acquire('key1', const Duration(seconds: 10));
      expect(guard.key, equals('key1'));
    });
  });

  group('release', () {
    test('ロックを解放できること', () async {
      final guard = await lock.acquire('key1', const Duration(seconds: 10));
      await lock.release(guard);
      expect(await lock.isLocked('key1'), isFalse);
    });

    test('トークンが一致しない場合に例外をスローすること', () async {
      await lock.acquire('key1', const Duration(seconds: 10));
      final fakeGuard = LockGuard(key: 'key1', token: 'wrong-token');
      expect(
        () => lock.release(fakeGuard),
        throwsA(isA<LockException>()),
      );
    });
  });

  group('isLocked', () {
    test('存在しないキーに対してfalseを返すこと', () async {
      expect(await lock.isLocked('unknown'), isFalse);
    });

    test('ロックが保持されている場合にtrueを返すこと', () async {
      await lock.acquire('key1', const Duration(seconds: 10));
      expect(await lock.isLocked('key1'), isTrue);
    });
  });
}
