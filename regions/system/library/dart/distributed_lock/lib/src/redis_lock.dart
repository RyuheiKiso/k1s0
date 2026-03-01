import 'dart:math';

import 'package:redis/redis.dart';

import 'lock.dart';

/// Lua script for safe release: only releases if token matches.
const _releaseScript = '''
if redis.call("GET", KEYS[1]) == ARGV[1] then
  return redis.call("DEL", KEYS[1])
else
  return 0
end
''';

class RedisDistributedLock implements DistributedLock {
  final Command _client;
  final _rng = Random.secure();

  RedisDistributedLock(this._client);

  String _generateToken() {
    return List.generate(
            16, (_) => _rng.nextInt(256).toRadixString(16).padLeft(2, '0'))
        .join();
  }

  @override
  Future<LockGuard> acquire(String key, Duration ttl) async {
    final token = _generateToken();
    final ms = ttl.inMilliseconds;
    // SET key token NX PX ms
    final result = await _client.send_object(['SET', key, token, 'NX', 'PX', ms.toString()]);
    if (result == null) {
      throw LockException('lock already held for key: $key');
    }
    return LockGuard(key: key, token: token);
  }

  @override
  Future<void> release(LockGuard guard) async {
    final result = await _client.send_object([
      'EVAL',
      _releaseScript,
      '1',
      guard.key,
      guard.token,
    ]);
    if (result == 0) {
      throw LockException(
          'lock not held or token mismatch for key: ${guard.key}');
    }
  }

  @override
  Future<bool> isLocked(String key) async {
    final result = await _client.send_object(['EXISTS', key]);
    return (result as int) > 0;
  }
}
