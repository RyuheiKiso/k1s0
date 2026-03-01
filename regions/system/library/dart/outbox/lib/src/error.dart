/// アウトボックス操作のエラーコード。
enum OutboxErrorCode { storeError, publishError, serializationError, notFound }

/// アウトボックス操作のエラー。
class OutboxError implements Exception {
  final OutboxErrorCode code;
  final String? message;
  final Object? cause;

  const OutboxError(this.code, {this.message, this.cause});

  @override
  String toString() {
    final buffer = StringBuffer('OutboxError(${code.name}');
    if (message != null) buffer.write(': $message');
    if (cause != null) buffer.write(' [cause: $cause]');
    buffer.write(')');
    return buffer.toString();
  }
}
