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

/// セッションクッキーインターセプター（モバイル向け）
/// Set-Cookie ヘッダーからセッションクッキーを抽出し、後続リクエストに自動付与する。
/// Flutter Web ではブラウザが Cookie を自動管理するため不要だが、
/// モバイルでは Dio が Cookie を扱わないため手動で管理する。
///
/// TODO(M-019): セキュリティ強化のため、セッション ID をメモリ上の String ではなく
/// flutter_secure_storage（AES-GCM 暗号化）で管理することを検討する。
/// 特に越獄済みデバイスやメモリダンプ攻撃に対する防御として有効。
/// 参考: https://pub.dev/packages/flutter_secure_storage
class SessionCookieInterceptor extends Interceptor {
  /// 保持中のセッション ID
  String? sessionId;

  /// セッションクッキー名
  final String cookieName;

  SessionCookieInterceptor({this.cookieName = 'k1s0_session'});

  @override
  void onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) {
    // セッション ID が保持されている場合はリクエストに Cookie ヘッダーを付与する
    if (sessionId != null) {
      final existing = options.headers['Cookie'] as String?;
      final sessionCookie = '$cookieName=$sessionId';
      options.headers['Cookie'] =
          existing != null ? '$existing; $sessionCookie' : sessionCookie;
    }
    handler.next(options);
  }

  @override
  void onResponse(
    Response response,
    ResponseInterceptorHandler handler,
  ) {
    // Set-Cookie ヘッダーからセッションクッキーを抽出して保持する
    _extractSessionCookie(response.headers);
    handler.next(response);
  }

  /// Set-Cookie ヘッダーからセッション ID を抽出する
  void _extractSessionCookie(Headers headers) {
    final setCookies = headers['set-cookie'];
    if (setCookies == null) return;
    for (final cookie in setCookies) {
      if (cookie.startsWith('$cookieName=')) {
        final value = cookie.split(';').first.substring(cookieName.length + 1);
        if (value.isNotEmpty) {
          sessionId = value;
        }
      }
    }
  }

  /// セッションをクリアする（ログアウト時に使用）
  void clearSession() {
    sessionId = null;
  }
}

class ApiClient {
  ApiClient._();

  static Dio create({
    required String baseUrl,
    Duration connectTimeout = const Duration(seconds: 30),
    Duration receiveTimeout = const Duration(seconds: 30),
    CsrfTokenProvider? csrfTokenProvider,
    SessionCookieInterceptor? sessionCookieInterceptor,
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

    // セッションクッキーインターセプター（モバイルでのセッション管理に必要）
    if (sessionCookieInterceptor != null) {
      dio.interceptors.add(sessionCookieInterceptor);
    }

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
