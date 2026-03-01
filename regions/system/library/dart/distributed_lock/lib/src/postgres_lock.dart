import 'dart:math';

import 'package:postgres/postgres.dart';

import 'lock.dart';

class PostgresDistributedLock implements DistributedLock {
  final Connection _conn;
  final _rng = Random.secure();

  PostgresDistributedLock(this._conn);

  String _generateToken() {
    return List.generate(
            16, (_) => _rng.nextInt(256).toRadixString(16).padLeft(2, '0'))
        .join();
  }

  /// Returns the advisory lock key as an integer hash of the key string.
  /// Uses hashtext equivalent: we use Dart's string hash and mask to 32-bit.
  int _lockKey(String key) => key.hashCode & 0x7fffffff;

  @override
  Future<LockGuard> acquire(String key, Duration ttl) async {
    final lockKey = _lockKey(key);
    final rows = await _conn.execute(
      Sql.named('SELECT pg_try_advisory_lock(@key)'),
      parameters: {'key': lockKey},
    );
    final acquired = rows.first[0] as bool? ?? false;
    if (!acquired) {
      throw LockException('lock already held for key: $key');
    }
    final token = _generateToken();
    return LockGuard(key: key, token: token);
  }

  @override
  Future<void> release(LockGuard guard) async {
    final lockKey = _lockKey(guard.key);
    final rows = await _conn.execute(
      Sql.named('SELECT pg_advisory_unlock(@key)'),
      parameters: {'key': lockKey},
    );
    final released = rows.first[0] as bool? ?? false;
    if (!released) {
      throw LockException(
          'lock not held or token mismatch for key: ${guard.key}');
    }
  }

  @override
  Future<bool> isLocked(String key) async {
    final lockKey = _lockKey(key);
    final rows = await _conn.execute(
      Sql.named(
          'SELECT COUNT(*) FROM pg_locks WHERE locktype = \'advisory\' AND objid = @key AND granted = true'),
      parameters: {'key': lockKey},
    );
    final count = rows.first[0] as int? ?? 0;
    return count > 0;
  }
}
