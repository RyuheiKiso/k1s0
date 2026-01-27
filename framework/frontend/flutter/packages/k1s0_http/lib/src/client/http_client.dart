import 'package:dio/dio.dart';

import '../error/api_error.dart';
import '../interceptors/auth_interceptor.dart';
import '../interceptors/error_interceptor.dart';
import '../interceptors/logging_interceptor.dart';
import '../interceptors/trace_interceptor.dart';
import '../types/request_options.dart';
import '../types/response.dart';
import 'http_client_config.dart';

/// k1s0 HTTP client
///
/// A Dio-based HTTP client with built-in support for:
/// - Authentication token management
/// - Request/response logging
/// - Error handling with ProblemDetails support
/// - OpenTelemetry trace context propagation
/// - Retry with exponential backoff
class K1s0HttpClient {
  /// Creates a k1s0 HTTP client
  K1s0HttpClient({
    required HttpClientConfig config,
    TokenProvider? tokenProvider,
    ErrorCallback? onError,
    AuthErrorCallback? onAuthError,
    void Function(String)? logger,
  }) : _config = config {
    _dio = Dio(
      BaseOptions(
        baseUrl: config.baseUrl,
        connectTimeout: config.connectTimeout,
        receiveTimeout: config.timeout,
        sendTimeout: config.timeout,
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json',
          ...config.defaultHeaders,
        },
        validateStatus: config.validateStatus ?? (status) => status != null && status < 400,
      ),
    );

    // Add interceptors in order
    _dio.interceptors.addAll([
      // Trace interceptor first to set trace ID
      TraceInterceptor(),

      // Auth interceptor
      if (tokenProvider != null)
        AuthInterceptor(tokenProvider: tokenProvider),

      // Logging interceptor
      if (config.logLevel != HttpLogLevel.none)
        LoggingInterceptor(
          logLevel: config.logLevel,
          logger: logger,
        ),

      // Error interceptor last
      ErrorInterceptor(
        errorCallback: onError,
        authErrorCallback: onAuthError,
      ),
    ]);
  }

  final HttpClientConfig _config;
  late final Dio _dio;

  /// Get the underlying Dio instance for advanced usage
  Dio get dio => _dio;

  /// GET request
  Future<K1s0Response<T>> get<T>(
    String path, {
    K1s0RequestOptions? options,
  }) async =>
      _request<T>(
        path,
        method: 'GET',
        options: options,
      );

  /// POST request
  Future<K1s0Response<T>> post<T>(
    String path, {
    Object? data,
    K1s0RequestOptions? options,
  }) async =>
      _request<T>(
        path,
        method: 'POST',
        data: data,
        options: options,
      );

  /// PUT request
  Future<K1s0Response<T>> put<T>(
    String path, {
    Object? data,
    K1s0RequestOptions? options,
  }) async =>
      _request<T>(
        path,
        method: 'PUT',
        data: data,
        options: options,
      );

  /// PATCH request
  Future<K1s0Response<T>> patch<T>(
    String path, {
    Object? data,
    K1s0RequestOptions? options,
  }) async =>
      _request<T>(
        path,
        method: 'PATCH',
        data: data,
        options: options,
      );

  /// DELETE request
  Future<K1s0Response<T>> delete<T>(
    String path, {
    Object? data,
    K1s0RequestOptions? options,
  }) async =>
      _request<T>(
        path,
        method: 'DELETE',
        data: data,
        options: options,
      );

  /// Generic request method
  Future<K1s0Response<T>> _request<T>(
    String path, {
    required String method,
    Object? data,
    K1s0RequestOptions? options,
  }) async {
    final retryPolicy = (options?.retry ?? false)
        ? _config.retryPolicy
        : RetryPolicy.none;

    var attempt = 0;
    ApiError? lastError;

    while (true) {
      try {
        final response = await _executeRequest<T>(
          path,
          method: method,
          data: data,
          options: options,
        );
        return response;
      } on DioException catch (e) {
        final traceId = e.requestOptions.extra['traceId'] as String?;
        lastError = ApiError.fromDioError(e, traceId);

        // Check if we should retry
        if (!_shouldRetry(e, retryPolicy, attempt)) {
          throw lastError;
        }

        attempt++;

        // Wait before retrying
        final delay = retryPolicy.delayForAttempt(attempt);
        await Future<void>.delayed(delay);
      }
    }
  }

  Future<K1s0Response<T>> _executeRequest<T>(
    String path, {
    required String method,
    Object? data,
    K1s0RequestOptions? options,
  }) async {
    final response = await _dio.request<T>(
      path,
      data: data,
      queryParameters: options?.queryParameters,
      options: Options(
        method: method,
        headers: options?.headers,
        sendTimeout: options?.timeout != null
            ? Duration(milliseconds: options!.timeout!)
            : null,
        receiveTimeout: options?.timeout != null
            ? Duration(milliseconds: options!.timeout!)
            : null,
        extra: {
          'skipAuth': options?.skipAuth ?? false,
          if (options?.traceId != null) 'traceId': options!.traceId,
          ...?options?.extra,
        },
      ),
    );

    return K1s0Response.fromDioResponse(response);
  }

  bool _shouldRetry(DioException error, RetryPolicy policy, int attempt) {
    if (attempt >= policy.maxAttempts) {
      return false;
    }

    switch (error.type) {
      case DioExceptionType.connectionTimeout:
      case DioExceptionType.sendTimeout:
      case DioExceptionType.receiveTimeout:
        return policy.retryOnTimeout;

      case DioExceptionType.connectionError:
        return policy.retryOnConnectionError;

      case DioExceptionType.badResponse:
        final statusCode = error.response?.statusCode;
        return statusCode != null && policy.retryStatusCodes.contains(statusCode);

      default:
        return false;
    }
  }

  /// Close the client and release resources
  void close() {
    _dio.close();
  }
}

/// Create an HTTP client with the given configuration
K1s0HttpClient createK1s0HttpClient({
  required String baseUrl,
  Duration timeout = const Duration(seconds: 30),
  Duration connectTimeout = const Duration(seconds: 10),
  Map<String, String> defaultHeaders = const {},
  HttpLogLevel logLevel = HttpLogLevel.basic,
  TokenProvider? tokenProvider,
  ErrorCallback? onError,
  AuthErrorCallback? onAuthError,
  void Function(String)? logger,
  RetryPolicy retryPolicy = RetryPolicy.none,
}) =>
    K1s0HttpClient(
      config: HttpClientConfig(
        baseUrl: baseUrl,
        timeout: timeout,
        connectTimeout: connectTimeout,
        defaultHeaders: defaultHeaders,
        logLevel: logLevel,
        retryPolicy: retryPolicy,
      ),
      tokenProvider: tokenProvider,
      onError: onError,
      onAuthError: onAuthError,
      logger: logger,
    );

/// Factory for creating HTTP clients
///
/// This class provides backward compatibility.
/// Consider using the top-level function [createK1s0HttpClient] instead.
class K1s0HttpClientFactory {
  K1s0HttpClientFactory._();

  /// Create an HTTP client with the given configuration
  static K1s0HttpClient create({
    required String baseUrl,
    Duration timeout = const Duration(seconds: 30),
    Duration connectTimeout = const Duration(seconds: 10),
    Map<String, String> defaultHeaders = const {},
    HttpLogLevel logLevel = HttpLogLevel.basic,
    TokenProvider? tokenProvider,
    ErrorCallback? onError,
    AuthErrorCallback? onAuthError,
    void Function(String)? logger,
    RetryPolicy retryPolicy = RetryPolicy.none,
  }) =>
      createK1s0HttpClient(
        baseUrl: baseUrl,
        timeout: timeout,
        connectTimeout: connectTimeout,
        defaultHeaders: defaultHeaders,
        logLevel: logLevel,
        tokenProvider: tokenProvider,
        onError: onError,
        onAuthError: onAuthError,
        logger: logger,
        retryPolicy: retryPolicy,
      );
}
