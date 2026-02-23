class IdempotencyError implements Exception {
  final String message;
  final String code;

  const IdempotencyError(this.message, this.code);

  @override
  String toString() => 'IdempotencyError($code): $message';
}

class DuplicateKeyError implements Exception {
  final String key;

  const DuplicateKeyError(this.key);

  @override
  String toString() => 'DuplicateKeyError: duplicate key: $key';
}
