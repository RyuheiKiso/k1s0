class ValidationError implements Exception {
  final String field;
  final String message;

  const ValidationError(this.field, this.message);

  @override
  String toString() => 'ValidationError($field): $message';
}
