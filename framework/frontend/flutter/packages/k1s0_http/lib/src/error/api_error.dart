import 'package:dio/dio.dart';

import 'problem_details.dart';

/// API error classification
enum ApiErrorKind {
  /// Validation error (400)
  validation,

  /// Authentication error (401)
  authentication,

  /// Authorization error (403)
  authorization,

  /// Resource not found (404)
  notFound,

  /// Conflict error (409)
  conflict,

  /// Rate limit exceeded (429)
  rateLimit,

  /// Dependency failure (502/503)
  dependency,

  /// Temporary error / retryable (503)
  temporary,

  /// Timeout error
  timeout,

  /// Network error
  network,

  /// Connection error
  connection,

  /// Request cancelled
  cancelled,

  /// Unknown error
  unknown,
}

/// API error with RFC 7807 Problem Details support
class ApiError implements Exception {
  /// Creates an API error
  ApiError({
    required this.kind,
    required this.message,
    this.statusCode,
    this.errorCode,
    this.traceId,
    this.problemDetails,
    this.cause,
    this.stackTrace,
  });

  /// Creates an API error from a Dio error
  factory ApiError.fromDioError(DioException error, [String? traceId]) {
    switch (error.type) {
      case DioExceptionType.connectionTimeout:
      case DioExceptionType.sendTimeout:
      case DioExceptionType.receiveTimeout:
        return ApiError(
          kind: ApiErrorKind.timeout,
          message: 'Request timed out',
          traceId: traceId,
          cause: error,
        );

      case DioExceptionType.connectionError:
        return ApiError(
          kind: ApiErrorKind.connection,
          message: 'Connection failed',
          traceId: traceId,
          cause: error,
        );

      case DioExceptionType.cancel:
        return ApiError(
          kind: ApiErrorKind.cancelled,
          message: 'Request cancelled',
          traceId: traceId,
          cause: error,
        );

      case DioExceptionType.badResponse:
        return ApiError.fromResponse(error.response!, traceId);

      case DioExceptionType.badCertificate:
        return ApiError(
          kind: ApiErrorKind.network,
          message: 'Certificate verification failed',
          traceId: traceId,
          cause: error,
        );

      case DioExceptionType.unknown:
        return ApiError(
          kind: ApiErrorKind.network,
          message: error.message ?? 'Network error',
          traceId: traceId,
          cause: error,
        );
    }
  }

  /// Creates an API error from a Dio response
  factory ApiError.fromResponse(Response<dynamic> response, [String? traceId]) {
    final statusCode = response.statusCode ?? 0;
    final kind = _mapStatusToKind(statusCode);

    // Try to parse Problem Details from response
    ProblemDetails? problemDetails;
    String? errorCode;
    String message;
    var responseTraceId = traceId;

    if (response.data is Map<String, dynamic>) {
      final data = response.data as Map<String, dynamic>;

      // Check if it's a Problem Details response
      if (data.containsKey('error_code') || data.containsKey('type')) {
        try {
          problemDetails = ProblemDetails.fromJson(data);
          errorCode = problemDetails.errorCode;
          message = problemDetails.detail ?? problemDetails.title;
          responseTraceId = problemDetails.traceId ?? traceId;
        } on Exception {
          message = data['message']?.toString() ??
              data['error']?.toString() ??
              _getDefaultMessage(kind);
          errorCode = data['error_code']?.toString();
        }
      } else {
        message = data['message']?.toString() ??
            data['error']?.toString() ??
            _getDefaultMessage(kind);
        errorCode = data['error_code']?.toString();
      }
    } else if (response.data is String && (response.data as String).isNotEmpty) {
      message = response.data as String;
    } else {
      message = _getDefaultMessage(kind);
    }

    return ApiError(
      kind: kind,
      message: message,
      statusCode: statusCode,
      errorCode: errorCode,
      traceId: responseTraceId,
      problemDetails: problemDetails,
    );
  }

  /// Error classification
  final ApiErrorKind kind;

  /// Human-readable error message
  final String message;

  /// HTTP status code (if applicable)
  final int? statusCode;

  /// Error code from the server
  final String? errorCode;

  /// Trace ID for debugging
  final String? traceId;

  /// Full Problem Details from the server
  final ProblemDetails? problemDetails;

  /// The original exception that caused this error
  final Object? cause;

  /// Stack trace from the original exception
  final StackTrace? stackTrace;

  /// Whether this error requires authentication
  bool get requiresAuthentication => kind == ApiErrorKind.authentication;

  /// Whether this error is retryable
  bool get isRetryable {
    switch (kind) {
      case ApiErrorKind.temporary:
      case ApiErrorKind.dependency:
      case ApiErrorKind.timeout:
      case ApiErrorKind.network:
      case ApiErrorKind.connection:
        return true;
      default:
        return false;
    }
  }

  /// Get field errors from Problem Details
  List<FieldError> get fieldErrors => problemDetails?.errors ?? [];

  /// Whether this is a validation error with field errors
  bool get hasFieldErrors => fieldErrors.isNotEmpty;

  @override
  String toString() {
    final buffer = StringBuffer('ApiError: $message');
    if (errorCode != null) {
      buffer.write(' (code: $errorCode)');
    }
    if (statusCode != null) {
      buffer.write(' [HTTP $statusCode]');
    }
    if (traceId != null) {
      buffer.write(' [trace: $traceId]');
    }
    return buffer.toString();
  }

  static ApiErrorKind _mapStatusToKind(int status) {
    if (status == 400) return ApiErrorKind.validation;
    if (status == 401) return ApiErrorKind.authentication;
    if (status == 403) return ApiErrorKind.authorization;
    if (status == 404) return ApiErrorKind.notFound;
    if (status == 409) return ApiErrorKind.conflict;
    if (status == 429) return ApiErrorKind.rateLimit;
    if (status == 502 || status == 503) return ApiErrorKind.dependency;
    if (status == 504) return ApiErrorKind.timeout;
    if (status >= 500) return ApiErrorKind.temporary;
    return ApiErrorKind.unknown;
  }

  static String _getDefaultMessage(ApiErrorKind kind) {
    switch (kind) {
      case ApiErrorKind.validation:
        return 'Invalid input';
      case ApiErrorKind.authentication:
        return 'Authentication required';
      case ApiErrorKind.authorization:
        return 'Access denied';
      case ApiErrorKind.notFound:
        return 'Resource not found';
      case ApiErrorKind.conflict:
        return 'Resource conflict';
      case ApiErrorKind.rateLimit:
        return 'Too many requests';
      case ApiErrorKind.dependency:
        return 'Service temporarily unavailable';
      case ApiErrorKind.temporary:
        return 'Temporary error, please retry';
      case ApiErrorKind.timeout:
        return 'Request timed out';
      case ApiErrorKind.network:
        return 'Network error';
      case ApiErrorKind.connection:
        return 'Connection failed';
      case ApiErrorKind.cancelled:
        return 'Request cancelled';
      case ApiErrorKind.unknown:
        return 'An unexpected error occurred';
    }
  }
}

/// Extension methods for ApiError
extension ApiErrorExtension on ApiError {
  /// Get a user-friendly error message in Japanese
  String get localizedMessage {
    switch (kind) {
      case ApiErrorKind.validation:
        return '入力内容に問題があります。内容を確認してください。';
      case ApiErrorKind.authentication:
        return '認証が必要です。ログインしてください。';
      case ApiErrorKind.authorization:
        return 'この操作を行う権限がありません。';
      case ApiErrorKind.notFound:
        return '指定されたリソースが見つかりません。';
      case ApiErrorKind.conflict:
        return 'データが競合しています。最新の状態を確認してください。';
      case ApiErrorKind.rateLimit:
        return 'リクエストが多すぎます。しばらくしてから再試行してください。';
      case ApiErrorKind.dependency:
        return 'サービスに一時的な問題が発生しています。';
      case ApiErrorKind.temporary:
        return 'サービスが一時的に利用できません。しばらくしてから再試行してください。';
      case ApiErrorKind.timeout:
        return 'リクエストがタイムアウトしました。';
      case ApiErrorKind.network:
        return 'ネットワークに接続できません。接続を確認してください。';
      case ApiErrorKind.connection:
        return 'サーバーに接続できません。';
      case ApiErrorKind.cancelled:
        return 'リクエストがキャンセルされました。';
      case ApiErrorKind.unknown:
        return '予期しないエラーが発生しました。';
    }
  }
}
