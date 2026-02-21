class ServiceAuthError implements Exception {
  final String message;
  final Object? cause;

  const ServiceAuthError(this.message, {this.cause});

  @override
  String toString() => cause != null
      ? 'ServiceAuthError: $message (cause: $cause)'
      : 'ServiceAuthError: $message';
}
