class ComponentError implements Exception {
  final String component;
  final String operation;
  final String message;
  final Object? cause;

  const ComponentError({
    required this.component,
    required this.operation,
    required this.message,
    this.cause,
  });

  @override
  String toString() {
    if (cause != null) {
      return '[$component] $operation: $message: $cause';
    }
    return '[$component] $operation: $message';
  }
}

class ETagMismatchError implements Exception {
  final String key;
  final String expected;
  final String actual;

  const ETagMismatchError({
    required this.key,
    required this.expected,
    required this.actual,
  });

  @override
  String toString() => 'ETag mismatch for key "$key": expected "$expected", got "$actual"';
}
