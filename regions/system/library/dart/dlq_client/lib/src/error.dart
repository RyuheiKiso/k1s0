/// DLQ クライアントエラー。
class DlqException implements Exception {
  final String message;
  final int? statusCode;

  const DlqException(this.message, {this.statusCode});

  @override
  String toString() => statusCode != null
      ? 'DlqException($statusCode): $message'
      : 'DlqException: $message';
}
