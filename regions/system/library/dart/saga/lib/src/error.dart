/// Saga クライアントエラー。
class SagaException implements Exception {
  final String message;
  final int? statusCode;

  const SagaException(this.message, {this.statusCode});

  @override
  String toString() => statusCode != null
      ? 'SagaException($statusCode): $message'
      : 'SagaException: $message';
}
