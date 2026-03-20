
import 'package:redis/redis.dart';

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
  // 指定キーの有効期限をミリ秒単位で更新する
  Future<bool> expire(String key, int ttlMs);
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

  // 指定キーの有効期限を更新する。キーが存在しないまたは期限切れの場合はfalseを返す
  @override
  Future<bool> expire(String key, int ttlMs) async {
    final entry = _entries[key];
    if (entry == null) return false;
    if (entry.isExpired) {
      _entries.remove(key);
      return false;
    }
    _entries[key] = _Entry(
      entry.value,
      DateTime.now().millisecondsSinceEpoch + ttlMs,
    );
    return true;
  }
}

class RedisCacheClient implements CacheClient {
  final Command _command;
  final String keyPrefix;

  RedisCacheClient(this._command, {this.keyPrefix = ''});

  static Future<RedisCacheClient> connect(
    String host,
    int port, {
    String keyPrefix = '',
    int? db,
  }) async {
    final connection = RedisConnection();
    final command = await connection.connect(host, port);
    if (db != null) {
      await command.send_object(['SELECT', db]);
    }
    return RedisCacheClient(command, keyPrefix: keyPrefix);
  }

  @override
  Future<String?> get(String key) async {
    final result = await _command.send_object(['GET', _prefixedKey(key)]);
    if (result == null) return null;
    return result.toString();
  }

  @override
  Future<void> set(String key, String value, {int? ttlMs}) async {
    if (ttlMs != null) {
      await _command.send_object(['PSETEX', _prefixedKey(key), ttlMs, value]);
      return;
    }
    await _command.send_object(['SET', _prefixedKey(key), value]);
  }

  @override
  Future<bool> delete(String key) async {
    final result = await _command.send_object(['DEL', _prefixedKey(key)]);
    return (result as int) > 0;
  }

  @override
  Future<bool> exists(String key) async {
    final result = await _command.send_object(['EXISTS', _prefixedKey(key)]);
    return (result as int) > 0;
  }

  @override
  Future<bool> setNX(String key, String value, int ttlMs) async {
    final result = await _command.send_object([
      'SET',
      _prefixedKey(key),
      value,
      'PX',
      ttlMs,
      'NX',
    ]);
    return result == 'OK';
  }

  // PEXPIREコマンドでキーの有効期限をミリ秒単位で設定する
  @override
  Future<bool> expire(String key, int ttlMs) async {
    final result =
        await _command.send_object(['PEXPIRE', _prefixedKey(key), ttlMs]);
    return (result as int) > 0;
  }

  String _prefixedKey(String key) {
    return keyPrefix.isEmpty ? key : '$keyPrefix:$key';
  }
}
