class MessagingError implements Exception {
  final String op;
  final Object? cause;

  const MessagingError(this.op, {this.cause});

  @override
  String toString() => cause != null
      ? 'MessagingError($op): $cause'
      : 'MessagingError($op)';
}
