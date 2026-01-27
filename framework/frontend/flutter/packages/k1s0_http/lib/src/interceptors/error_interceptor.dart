import 'package:dio/dio.dart';

import '../error/api_error.dart';

/// Callback for handling API errors
typedef ErrorCallback = void Function(ApiError error);

/// Callback for handling authentication errors
typedef AuthErrorCallback = void Function(ApiError error);

/// Interceptor that converts Dio errors to ApiError
class ErrorInterceptor extends Interceptor {
  /// Creates an error interceptor
  ErrorInterceptor({
    this.errorCallback,
    this.authErrorCallback,
  });

  /// Callback for all errors
  final ErrorCallback? errorCallback;

  /// Callback for authentication errors (401)
  final AuthErrorCallback? authErrorCallback;

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    final traceId = err.requestOptions.extra['traceId'] as String?;
    final apiError = ApiError.fromDioError(err, traceId);

    // Notify error callbacks
    errorCallback?.call(apiError);

    if (apiError.requiresAuthentication) {
      authErrorCallback?.call(apiError);
    }

    // Reject with the original error - let callers handle ApiError conversion
    handler.next(err);
  }
}
