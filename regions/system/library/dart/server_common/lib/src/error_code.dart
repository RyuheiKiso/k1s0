/// 構造化エラーコードクラス。
/// エラーコードは `SYS_{SERVICE}_{ERROR}` パターンに従う。
class ErrorCode {
  /// エラーコード文字列
  final String value;

  const ErrorCode(this.value);

  /// 指定サービスの「Not Found」エラーコードを生成する。
  factory ErrorCode.notFound(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_NOT_FOUND');
  }

  /// 指定サービスの「Validation Failed」エラーコードを生成する。
  factory ErrorCode.validation(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_VALIDATION_FAILED');
  }

  /// 指定サービスの「Internal Error」エラーコードを生成する。
  factory ErrorCode.internal(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_INTERNAL_ERROR');
  }

  /// 指定サービスの「Unauthorized」エラーコードを生成する。
  factory ErrorCode.unauthorized(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_UNAUTHORIZED');
  }

  /// 指定サービスの「Permission Denied」エラーコードを生成する。
  factory ErrorCode.forbidden(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_PERMISSION_DENIED');
  }

  /// 指定サービスの「Conflict」エラーコードを生成する。
  factory ErrorCode.conflict(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_CONFLICT');
  }

  /// 指定サービスの「Business Rule Violation」エラーコードを生成する。
  factory ErrorCode.unprocessable(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_BUSINESS_RULE_VIOLATION');
  }

  /// 指定サービスの「Rate Exceeded」エラーコードを生成する。
  factory ErrorCode.rateExceeded(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_RATE_EXCEEDED');
  }

  /// 指定サービスの「Service Unavailable」エラーコードを生成する。
  factory ErrorCode.serviceUnavailable(String service) {
    return ErrorCode('SYS_${service.toUpperCase()}_SERVICE_UNAVAILABLE');
  }

  /// ビジネス層サービスの「Not Found」エラーコードを生成する。
  factory ErrorCode.bizNotFound(String service) {
    return ErrorCode('BIZ_${service.toUpperCase()}_NOT_FOUND');
  }

  /// ビジネス層サービスの「Validation Failed」エラーコードを生成する。
  factory ErrorCode.bizValidation(String service) {
    return ErrorCode('BIZ_${service.toUpperCase()}_VALIDATION_FAILED');
  }

  /// サービス層サービスの「Not Found」エラーコードを生成する。
  factory ErrorCode.svcNotFound(String service) {
    return ErrorCode('SVC_${service.toUpperCase()}_NOT_FOUND');
  }

  /// サービス層サービスの「Validation Failed」エラーコードを生成する。
  factory ErrorCode.svcValidation(String service) {
    return ErrorCode('SVC_${service.toUpperCase()}_VALIDATION_FAILED');
  }

  @override
  String toString() => value;

  @override
  bool operator ==(Object other) =>
      other is ErrorCode && other.value == value;

  @override
  int get hashCode => value.hashCode;
}
