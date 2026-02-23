import 'dart:math';

class LockGuard {
  final String key;
  final String token;

  const LockGuard({required this.key, required this.token});
}

class LockException implements Exception {
  final String message;

  const LockException(this.message);

  @override
  String toString() => 'LockException: $message';
}

abstract class DistributedLock {
  Future<LockGuard> acquire(String key, Duration ttl);
  Future<void> release(LockGuard guard);
  Future<bool> isLocked(String key);
}

class _LockEntry {
  final String token;
  final DateTime expiresAt;

  _LockEntry(this.token, this.expiresAt);

  bool get isExpired => DateTime.now().isAfter(expiresAt);
}

class InMemoryDistributedLock implements DistributedLock {
  final Map<String, _LockEntry> _locks = {};
  final _rng = Random.secure();

  String _generateToken() {
    return List.generate(16, (_) => _rng.nextInt(256).toRadixString(16).padLeft(2, '0')).join();
  }

  @override
  Future<LockGuard> acquire(String key, Duration ttl) async {
    final existing = _locks[key];
    if (existing != null && !existing.isExpired) {
      throw LockException('lock already held for key: $key');
    }

    final token = _generateToken();
    _locks[key] = _LockEntry(token, DateTime.now().add(ttl));
    return LockGuard(key: key, token: token);
  }

  @override
  Future<void> release(LockGuard guard) async {
    final existing = _locks[guard.key];
    if (existing == null || existing.token != guard.token) {
      throw LockException('lock not held or token mismatch for key: ${guard.key}');
    }
    _locks.remove(guard.key);
  }

  @override
  Future<bool> isLocked(String key) async {
    final existing = _locks[key];
    if (existing == null || existing.isExpired) {
      _locks.remove(key);
      return false;
    }
    return true;
  }
}
