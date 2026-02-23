class CacheError implements Exception {
  final String message;
  final String code;

  const CacheError(this.message, this.code);

  @override
  String toString() => 'CacheError($code): $message';
}
