import 'package:dio/dio.dart';

class ApiClient {
  ApiClient._();

  static Dio create({
    required String baseUrl,
    Duration connectTimeout = const Duration(seconds: 30),
    Duration receiveTimeout = const Duration(seconds: 30),
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
    dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) {
          // Cookie ベース認証のための設定
          // CSRF トークンがある場合はヘッダーに追加
          handler.next(options);
        },
        onError: (error, handler) {
          handler.next(error);
        },
      ),
    );

    return dio;
  }
}
