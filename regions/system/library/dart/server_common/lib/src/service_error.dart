import 'error_code.dart';
import 'error_detail.dart';
import 'error_response.dart';

/// サービスエラーの種別。
/// HTTP ステータスコードに対応する。
enum ServiceErrorType {
  /// 404 Not Found
  notFound,

  /// 400 Bad Request
  badRequest,

  /// 401 Unauthorized
  unauthorized,

  /// 403 Forbidden
  forbidden,

  /// 409 Conflict
  conflict,

  /// 422 Unprocessable Entity
  unprocessableEntity,

  /// 429 Too Many Requests
  tooManyRequests,

  /// 500 Internal Server Error
  internal,

  /// 503 Service Unavailable
  serviceUnavailable,
}

/// 高レベルのサービスエラー。
/// HTTP ステータスコードに対応する構造化エラーコードとメッセージを保持する。
class ServiceError implements Exception {
  /// エラー種別
  final ServiceErrorType type;

  /// 構造化エラーコード
  final ErrorCode code;

  /// エラーメッセージ
  final String message;

  /// フィールドレベルのエラー詳細（オプション）
  final List<ErrorDetail> details;

  const ServiceError({
    required this.type,
    required this.code,
    required this.message,
    this.details = const [],
  });

  /// 指定サービスの NotFound エラーを生成する。
  factory ServiceError.notFound(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.notFound,
      code: ErrorCode.notFound(service),
      message: message,
    );
  }

  /// 指定サービスの BadRequest エラーを生成する。
  factory ServiceError.badRequest(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.badRequest,
      code: ErrorCode.validation(service),
      message: message,
    );
  }

  /// 指定サービスの Unauthorized エラーを生成する。
  factory ServiceError.unauthorized(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.unauthorized,
      code: ErrorCode.unauthorized(service),
      message: message,
    );
  }

  /// 指定サービスの Forbidden エラーを生成する。
  factory ServiceError.forbidden(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.forbidden,
      code: ErrorCode.forbidden(service),
      message: message,
    );
  }

  /// 指定サービスの Conflict エラーを生成する。
  factory ServiceError.conflict(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.conflict,
      code: ErrorCode.conflict(service),
      message: message,
    );
  }

  /// 指定サービスの UnprocessableEntity エラーを生成する。
  factory ServiceError.unprocessableEntity(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.unprocessableEntity,
      code: ErrorCode.unprocessable(service),
      message: message,
    );
  }

  /// 指定サービスの TooManyRequests エラーを生成する。
  factory ServiceError.tooManyRequests(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.tooManyRequests,
      code: ErrorCode.rateExceeded(service),
      message: message,
    );
  }

  /// 指定サービスの Internal エラーを生成する。
  factory ServiceError.internal(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.internal,
      code: ErrorCode.internal(service),
      message: message,
    );
  }

  /// 指定サービスの ServiceUnavailable エラーを生成する。
  factory ServiceError.serviceUnavailable(String service, String message) {
    return ServiceError(
      type: ServiceErrorType.serviceUnavailable,
      code: ErrorCode.serviceUnavailable(service),
      message: message,
    );
  }

  /// ErrorResponse に変換する。
  ErrorResponse toErrorResponse() {
    if (details.isNotEmpty) {
      return ErrorResponse.withDetails(code, message, details);
    }
    return ErrorResponse(code, message);
  }

  @override
  String toString() => 'ServiceError(${code.value}): $message';
}
