import 'package:dio/dio.dart';

/// CSRF トークンを提供するコールバック型
typedef CsrfTokenProvider = Future<String?> Function();

/// CSRF トークンインターセプター
class CsrfTokenInterceptor extends Interceptor {
  CsrfTokenInterceptor({required this.tokenProvider});

  final CsrfTokenProvider tokenProvider;

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    final token = await tokenProvider();
    if (token != null && token.isNotEmpty) {
      options.headers['X-CSRF-Token'] = token;
    }
    handler.next(options);
  }
}

class ApiClient {
  ApiClient._();

  static Dio create({
    required String baseUrl,
    Duration connectTimeout = const Duration(seconds: 30),
    Duration receiveTimeout = const Duration(seconds: 30),
    CsrfTokenProvider? csrfTokenProvider,
  }) {
    final dio = Dio(
      BaseOptions(
        baseUrl: baseUrl,
        connectTimeout: connectTimeout,
        receiveTimeout: receiveTimeout,
        headers: {
          'Content-Type': 'application/json',
        },
      ),
    );

    // CSRF トークンインターセプター
    if (csrfTokenProvider != null) {
      dio.interceptors.add(
        CsrfTokenInterceptor(tokenProvider: csrfTokenProvider),
      );
    }

    // エラーハンドリングインターセプター
    dio.interceptors.add(
      InterceptorsWrapper(
        onError: (error, handler) {
          handler.next(error);
        },
      ),
    );

    return dio;
  }
}
