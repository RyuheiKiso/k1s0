import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

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
/// セッション ID は flutter_secure_storage（AES-GCM 暗号化）で管理し、
/// 越獄済みデバイスやメモリダンプ攻撃に対する防御を行う。（M-019 対応済み）
/// 参考: https://pub.dev/packages/flutter_secure_storage
class SessionCookieInterceptor extends Interceptor {
  /// flutter_secure_storage を使用した暗号化ストレージ（AES-GCM）
  final FlutterSecureStorage _storage;

  /// セッションクッキー名
  final String cookieName;

  /// セキュアストレージのキー名
  static const _sessionIdKey = 'session_id';

  // コンストラクタでセキュアストレージとクッキー名を初期化する
  SessionCookieInterceptor({
    this.cookieName = 'k1s0_session',
    FlutterSecureStorage? storage,
  }) : _storage = storage ?? const FlutterSecureStorage();

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    // セキュアストレージからセッション ID を非同期で読み出し、リクエストに Cookie ヘッダーを付与する
    final sessionId = await _storage.read(key: _sessionIdKey);
    if (sessionId != null) {
      final existing = options.headers['Cookie'] as String?;
      final sessionCookie = '$cookieName=$sessionId';
      options.headers['Cookie'] =
          existing != null ? '$existing; $sessionCookie' : sessionCookie;
    }
    handler.next(options);
  }

  @override
  Future<void> onResponse(
    Response response,
    ResponseInterceptorHandler handler,
  ) async {
    // Set-Cookie ヘッダーからセッションクッキーを抽出してセキュアストレージに保存する
    await _extractSessionCookie(response.headers);
    handler.next(response);
  }

  /// Set-Cookie ヘッダーからセッション ID を抽出し、セキュアストレージに書き込む
  Future<void> _extractSessionCookie(Headers headers) async {
    final setCookies = headers['set-cookie'];
    if (setCookies == null) return;
    for (final cookie in setCookies) {
      if (cookie.startsWith('$cookieName=')) {
        final value = cookie.split(';').first.substring(cookieName.length + 1);
        if (value.isNotEmpty) {
          // セッション ID を AES-GCM 暗号化ストレージに書き込む
          await _storage.write(key: _sessionIdKey, value: value);
        }
      }
    }
  }

  /// セッションをクリアする（ログアウト時に使用）
  Future<void> clearSession() async {
    // セキュアストレージからセッション ID を削除する
    await _storage.delete(key: _sessionIdKey);
  }
}

/// 401 Unauthorized を検出した際に呼び出すコールバック型
/// 認証状態のリセット処理を呼び出し元が注入できるようにする
typedef UnauthorizedCallback = void Function();

class ApiClient {
  ApiClient._();

  static Dio create({
    required String baseUrl,
    Duration connectTimeout = const Duration(seconds: 30),
    Duration receiveTimeout = const Duration(seconds: 30),
    CsrfTokenProvider? csrfTokenProvider,
    SessionCookieInterceptor? sessionCookieInterceptor,
    // 401 Unauthorized 発生時に認証状態をリセットするコールバック
    UnauthorizedCallback? onUnauthorized,
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
    // 401 Unauthorized を検出した場合はセッションをクリアして認証状態をリセットする
    dio.interceptors.add(
      InterceptorsWrapper(
        onError: (error, handler) {
          // 401 Unauthorized はセッション切れまたは未認証を示すため、認証状態をリセットする
          if (error.response?.statusCode == 401 && onUnauthorized != null) {
            onUnauthorized();
          }
          handler.next(error);
        },
      ),
    );

    return dio;
  }
}
