import 'error.dart';
import 'record.dart';

abstract class IdempotencyStore {
  Future<IdempotencyRecord?> get(String key);
  Future<void> insert(IdempotencyRecord record);
  Future<void> update(
    String key,
    IdempotencyStatus status, {
    String? body,
    int? code,
  });
  Future<bool> delete(String key);
}

class InMemoryIdempotencyStore implements IdempotencyStore {
  final Map<String, IdempotencyRecord> _records = {};

  void _cleanupExpired() {
    final now = DateTime.now();
    _records.removeWhere(
      (_, record) =>
          record.expiresAt != null && record.expiresAt!.isBefore(now),
    );
  }

  @override
  Future<IdempotencyRecord?> get(String key) async {
    _cleanupExpired();
    return _records[key];
  }

  @override
  Future<void> insert(IdempotencyRecord record) async {
    _cleanupExpired();
    if (_records.containsKey(record.key)) {
      throw DuplicateKeyError(record.key);
    }
    _records[record.key] = IdempotencyRecord(
      key: record.key,
      status: record.status,
      responseBody: record.responseBody,
      statusCode: record.statusCode,
      createdAt: record.createdAt,
      expiresAt: record.expiresAt,
      completedAt: record.completedAt,
    );
  }

  @override
  Future<void> update(
    String key,
    IdempotencyStatus status, {
    String? body,
    int? code,
  }) async {
    final record = _records[key];
    if (record == null) {
      throw IdempotencyError('key not found: $key', 'NOT_FOUND');
    }
    record.status = status;
    record.responseBody = body;
    record.statusCode = code;
    record.completedAt = DateTime.now();
  }

  @override
  Future<bool> delete(String key) async {
    return _records.remove(key) != null;
  }
}
