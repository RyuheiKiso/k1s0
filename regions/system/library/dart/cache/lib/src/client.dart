import 'error.dart';

class _Entry {
  final String value;
  final int? expiresAt;

  _Entry(this.value, this.expiresAt);

  bool get isExpired =>
      expiresAt != null &&
      expiresAt! <= DateTime.now().millisecondsSinceEpoch;
}

abstract class CacheClient {
  Future<String?> get(String key);
  Future<void> set(String key, String value, {int? ttlMs});
  Future<bool> delete(String key);
  Future<bool> exists(String key);
  Future<bool> setNX(String key, String value, int ttlMs);
}

class InMemoryCacheClient implements CacheClient {
  final Map<String, _Entry> _entries = {};

  @override
  Future<String?> get(String key) async {
    final entry = _entries[key];
    if (entry == null) return null;
    if (entry.isExpired) {
      _entries.remove(key);
      return null;
    }
    return entry.value;
  }

  @override
  Future<void> set(String key, String value, {int? ttlMs}) async {
    _entries[key] = _Entry(
      value,
      ttlMs != null
          ? DateTime.now().millisecondsSinceEpoch + ttlMs
          : null,
    );
  }

  @override
  Future<bool> delete(String key) async {
    return _entries.remove(key) != null;
  }

  @override
  Future<bool> exists(String key) async {
    final entry = _entries[key];
    if (entry == null) return false;
    if (entry.isExpired) {
      _entries.remove(key);
      return false;
    }
    return true;
  }

  @override
  Future<bool> setNX(String key, String value, int ttlMs) async {
    final entry = _entries[key];
    if (entry != null && !entry.isExpired) return false;
    _entries[key] = _Entry(
      value,
      DateTime.now().millisecondsSinceEpoch + ttlMs,
    );
    return true;
  }
}
