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

/// セッション期限切れ時に呼び出すコールバック型
/// 認証状態のリセット処理を呼び出し元が注入できるようにする
typedef SessionExpiredCallback = void Function();

/// セッションクッキーインターセプター（モバイル向け）
/// Set-Cookie ヘッダーからセッションクッキーを抽出し、後続リクエストに自動付与する。
/// Flutter Web ではブラウザが Cookie を自動管理するため不要だが、
/// モバイルでは Dio が Cookie を扱わないため手動で管理する。
///
/// セッション ID と有効期限は flutter_secure_storage（AES-GCM 暗号化）で管理し、
/// 越獄済みデバイスやメモリダンプ攻撃に対する防御を行う。（M-019 対応済み）
/// リクエスト送信前に有効期限チェックを行い、期限切れセッションは自動的にクリアする。（FE-001 対応）
/// 参考: https://pub.dev/packages/flutter_secure_storage
class SessionCookieInterceptor extends Interceptor {
  /// flutter_secure_storage を使用した暗号化ストレージ（AES-GCM）
  final FlutterSecureStorage _storage;

  /// セッションクッキー名
  final String cookieName;

  /// セッション期限切れ時に呼び出すコールバック（認証状態リセット用）
  final SessionExpiredCallback? onSessionExpired;

  /// セキュアストレージのセッション ID キー名
  static const _sessionIdKey = 'session_id';

  /// セキュアストレージのセッション有効期限キー名（ISO 8601 形式で保存）
  static const _sessionExpiryKey = 'session_expiry';

  // コンストラクタでセキュアストレージとクッキー名、期限切れコールバックを初期化する
  SessionCookieInterceptor({
    this.cookieName = 'k1s0_session',
    FlutterSecureStorage? storage,
    this.onSessionExpired,
  }) : _storage = storage ?? const FlutterSecureStorage();

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    // セッションの有効期限を確認し、期限切れの場合はセッションをクリアして認証リセットを通知する
    if (await _isSessionExpired()) {
      await clearSession();
      onSessionExpired?.call();
      handler.next(options);
      return;
    }

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
    // Set-Cookie ヘッダーからセッションクッキーと有効期限を抽出してセキュアストレージに保存する
    await _extractSessionCookie(response.headers);
    handler.next(response);
  }

  // M-012 監査対応: セッション ID に Cookie ヘッダーを破損させる文字が含まれていないことを検証する
  // ';' が含まれると複数 Cookie として解析され、セキュリティ上の問題が発生する可能性がある
  bool _isValidSessionId(String sessionId) {
    // RFC 6265 に従い、Cookie 値として無効な文字を拒否する
    return sessionId.isNotEmpty &&
        !sessionId.contains(RegExp(r'[;\r\n\s]'));
  }

  /// セキュアストレージに保存された有効期限と現在時刻を比較してセッションが期限切れかどうかを返す
  /// 有効期限情報が存在しない場合はサーバー管理のセッションとみなして期限切れとはしない
  Future<bool> _isSessionExpired() async {
    // セッション ID がない場合は期限切れチェック不要
    final sessionId = await _storage.read(key: _sessionIdKey);
    if (sessionId == null) return false;

    // 有効期限情報が保存されていない場合はサーバー側で管理されているとみなす
    final expiryStr = await _storage.read(key: _sessionExpiryKey);
    if (expiryStr == null) return false;

    // ISO 8601 文字列をパースできない場合は安全側に倒して期限切れとみなす
    final expiry = DateTime.tryParse(expiryStr);
    if (expiry == null) return true;

    // 現在時刻が有効期限を過ぎていれば期限切れ
    return DateTime.now().isAfter(expiry);
  }

  /// Set-Cookie ヘッダーの max-age または expires 属性からセッションの有効期限を解析して返す
  /// max-age（秒数指定）を優先し、存在しない場合は expires（絶対日時）にフォールバックする
  /// どちらも存在しない場合は null を返す（サーバー管理のセッションクッキー）
  DateTime? _parseSessionExpiry(String cookieHeader) {
    // max-age を優先して解析する（現在時刻からの相対秒数）
    final maxAgeMatch =
        RegExp(r'max-age=(\d+)', caseSensitive: false).firstMatch(cookieHeader);
    if (maxAgeMatch != null) {
      final seconds = int.tryParse(maxAgeMatch.group(1)!);
      if (seconds != null) {
        return DateTime.now().add(Duration(seconds: seconds));
      }
    }

    // max-age がなければ expires の絶対日時を解析する（HTTP-date 形式）
    final expiresMatch =
        RegExp(r'expires=([^;]+)', caseSensitive: false).firstMatch(cookieHeader);
    if (expiresMatch != null) {
      // HttpDate.parse は dart:io のみ使用可能なため、手動で RFC 7231 形式をパースする
      // 例: "Thu, 01 Jan 2026 00:00:00 GMT"
      try {
        return _parseHttpDate(expiresMatch.group(1)!.trim());
      } catch (_) {
        // パース失敗は無視して null を返す
      }
    }

    return null;
  }

  /// RFC 7231 の HTTP-date 形式（例: "Thu, 01 Jan 2026 00:00:00 GMT"）を DateTime に変換する
  /// dart:io の HttpDate に依存せず、純粋な Dart で実装することでフルプラットフォーム互換を保つ
  DateTime? _parseHttpDate(String httpDate) {
    // 月名を数値にマッピングするテーブル
    const monthMap = {
      'Jan': 1, 'Feb': 2, 'Mar': 3, 'Apr': 4, 'May': 5, 'Jun': 6,
      'Jul': 7, 'Aug': 8, 'Sep': 9, 'Oct': 10, 'Nov': 11, 'Dec': 12,
    };

    // カンマ・空白で分割して空文字を除去する
    // 例: "Thu, 01 Jan 2026 00:00:00 GMT" → ["Thu", "01", "Jan", "2026", "00:00:00", "GMT"]
    // 例: "01 Jan 2026 00:00:00 GMT" → ["01", "Jan", "2026", "00:00:00", "GMT"]
    final normalized =
        httpDate.split(RegExp(r'[\s,]+')).where((p) => p.isNotEmpty).toList();

    // 先頭トークンが月名マップに含まれない場合は曜日（例: "Thu"）とみなしてオフセットをずらす
    // normalized[0] が "Thu" のような曜日なら normalized[1] が日付開始位置
    // normalized[0] が "01" のような数値なら offset 不要
    final hasDayOfWeek =
        normalized.isNotEmpty && !monthMap.containsKey(normalized[0]) &&
        int.tryParse(normalized[0]) == null;
    final offset = hasDayOfWeek ? 1 : 0;

    // 最低 "day month year time" の 4 トークンが必要
    if (normalized.length < 4 + offset) return null;

    final day = int.tryParse(normalized[0 + offset]);
    final month = monthMap[normalized[1 + offset]];
    final year = int.tryParse(normalized[2 + offset]);
    final timeParts = normalized[3 + offset].split(':');
    if (day == null || month == null || year == null || timeParts.length != 3) {
      return null;
    }

    final hour = int.tryParse(timeParts[0]);
    final minute = int.tryParse(timeParts[1]);
    final second = int.tryParse(timeParts[2]);
    if (hour == null || minute == null || second == null) return null;

    // HTTP-date は常に UTC で表現される
    return DateTime.utc(year, month, day, hour, minute, second);
  }

  /// Set-Cookie ヘッダーからセッション ID と有効期限を抽出し、セキュアストレージに書き込む
  Future<void> _extractSessionCookie(Headers headers) async {
    final setCookies = headers['set-cookie'];
    if (setCookies == null) return;
    for (final cookie in setCookies) {
      if (cookie.startsWith('$cookieName=')) {
        final value = cookie.split(';').first.substring(cookieName.length + 1);
        // M-012: セッション ID の文字検証を行い、不正な文字を含む場合は保存しない
        if (_isValidSessionId(value)) {
          // セッション ID を AES-GCM 暗号化ストレージに書き込む
          await _storage.write(key: _sessionIdKey, value: value);

          // max-age または expires から有効期限を解析してセキュアストレージに保存する
          // 有効期限が取得できない場合は期限情報を削除してサーバー管理に委ねる
          final expiry = _parseSessionExpiry(cookie);
          if (expiry != null) {
            await _storage.write(
              key: _sessionExpiryKey,
              value: expiry.toIso8601String(),
            );
          } else {
            await _storage.delete(key: _sessionExpiryKey);
          }
        }
      }
    }
  }

  /// セッションをクリアする（ログアウト時または期限切れ検出時に使用）
  Future<void> clearSession() async {
    // セッション ID と有効期限をセキュアストレージから削除する
    await _storage.delete(key: _sessionIdKey);
    await _storage.delete(key: _sessionExpiryKey);
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
