import 'package:dio/dio.dart';

/// Token provider interface
abstract class TokenProvider {
  /// Get a valid access token
  ///
  /// Should return null if no token is available.
  /// May refresh the token if it's expired.
  Future<String?> getToken();

  /// Called when the token is rejected by the server (401)
  Future<void> onTokenRejected();
}

/// Interceptor that adds authentication token to requests
class AuthInterceptor extends Interceptor {
  /// Creates an auth interceptor
  AuthInterceptor({
    required this.tokenProvider,
    this.tokenType = 'Bearer',
    this.headerName = 'Authorization',
  });

  /// Token provider
  final TokenProvider tokenProvider;

  /// Token type (e.g., "Bearer")
  final String tokenType;

  /// Header name for the token
  final String headerName;

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    // Skip auth if explicitly requested
    if (options.extra['skipAuth'] == true) {
      handler.next(options);
      return;
    }

    try {
      final token = await tokenProvider.getToken();

      if (token != null) {
        options.headers[headerName] = '$tokenType $token';
      }

      handler.next(options);
    } catch (e) {
      handler.reject(
        DioException(
          requestOptions: options,
          error: e,
          type: DioExceptionType.unknown,
          message: 'Failed to get authentication token',
        ),
      );
    }
  }

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    // Check if it's an authentication error
    if (err.response?.statusCode == 401) {
      await tokenProvider.onTokenRejected();
    }

    handler.next(err);
  }
}

/// Simple token provider that uses a callback
class CallbackTokenProvider implements TokenProvider {
  /// Creates a callback token provider
  CallbackTokenProvider({
    required this.getTokenCallback,
    this.onTokenRejectedCallback,
  });

  /// Callback to get the token
  final Future<String?> Function() getTokenCallback;

  /// Callback when token is rejected
  final Future<void> Function()? onTokenRejectedCallback;

  @override
  Future<String?> getToken() => getTokenCallback();

  @override
  Future<void> onTokenRejected() async {
    await onTokenRejectedCallback?.call();
  }
}
