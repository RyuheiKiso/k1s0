class OutboxError implements Exception {
  final String op;
  final Object? cause;

  const OutboxError(this.op, {this.cause});

  @override
  String toString() => cause != null
      ? 'OutboxError($op): $cause'
      : 'OutboxError($op)';
}
