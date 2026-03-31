import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import 'auth_state.dart';
import '../http/api_client.dart';

/// flutter_web_auth_2 の条件付きインポート用（モバイルでのみ使用）
/// flutter_web_auth_2 がインストールされていない環境でのコンパイルエラーを防ぐため、
/// 動的に呼び出す。
typedef WebAuthCallback = Future<String> Function({
  required String url,
  required String callbackUrlScheme,
});

/// モバイル向け OAuth 認証コールバック Provider
/// flutter_web_auth_2 の authenticate 関数を注入する。
/// テスト時やプラットフォーム別に差し替え可能にする。
final webAuthCallbackProvider = Provider<WebAuthCallback?>((_) => null);

/// BFF API ベース URL を提供する Provider
final authApiBaseUrlProvider = Provider<String>(
  (_) => '/bff',
);

/// モバイル OAuth のカスタム URL スキーム Provider
final authCallbackSchemeProvider = Provider<String>(
  (_) => 'k1s0',
);

/// セッションクッキーインターセプター Provider（モバイル専用）
/// テスト時は FlutterSecureStorage を使わないモックインスタンスに差し替え可能にする
final sessionCookieInterceptorProvider = Provider<SessionCookieInterceptor>(
  (_) => SessionCookieInterceptor(),
);

class AuthNotifier extends Notifier<AuthState> {
  late final Dio _apiClient;
  late final String _baseUrl;
  late final SessionCookieInterceptor _sessionCookieInterceptor;

  /// CSRF トークン（/auth/session または /auth/exchange のレスポンスから取得）
  String? _csrfToken;

  /// 現在保持している CSRF トークンを返す（サービスクライアントが csrfTokenProvider として使用）
  String? get csrfToken => _csrfToken;

  @override
  AuthState build() {
    _baseUrl = ref.read(authApiBaseUrlProvider);
    // Provider 経由でインターセプターを取得する（テスト時のモック差し替えを可能にする）
    _sessionCookieInterceptor = ref.read(sessionCookieInterceptorProvider);
    _apiClient = ApiClient.create(
      baseUrl: _baseUrl,
      // CSRF トークンを自動付与する
      csrfTokenProvider: () async => _csrfToken,
      // モバイルでのセッションクッキー管理
      sessionCookieInterceptor: kIsWeb ? null : _sessionCookieInterceptor,
      // 401 Unauthorized 検出時にクライアント側の認証状態をリセットするコールバック
      onUnauthorized: _handleUnauthorized,
    );
    _checkSession();
    return const AuthUnauthenticated();
  }

  /// 401 Unauthorized を検出した際にクライアント側のセッション情報をリセットする
  /// ApiClient から同期的に呼び出されるため、非同期処理は行わない
  void _handleUnauthorized() {
    _csrfToken = null;
    if (ref.mounted) {
      state = const AuthUnauthenticated();
    }
  }

  /// BFF の /auth/session エンドポイントでセッションを確認する
  /// 非同期操作後にプロバイダーが破棄されている可能性があるため、
  /// state 更新前に ref.mounted をチェックする（Riverpod 3.x 対応）
  Future<void> _checkSession() async {
    try {
      final response =
          await _apiClient.get<Map<String, dynamic>>('/auth/session');
      // 非同期操作完了後、プロバイダーが既に破棄されていれば何もしない
      if (!ref.mounted) return;
      final data = response.data;
      if (data != null && data['authenticated'] == true && data['id'] != null) {
        _csrfToken = data['csrf_token'] as String?;
        state = AuthAuthenticated(userId: data['id'] as String);
      }
    } catch (_) {
      // 非同期操作完了後、プロバイダーが既に破棄されていれば何もしない
      if (!ref.mounted) return;
      state = const AuthUnauthenticated();
    }
  }

  /// OAuth2/OIDC 認可コードフローを開始する
  /// Web: BFF のログイン URL にブラウザをリダイレクトする
  /// Mobile: flutter_web_auth_2 で OAuth フローを実行し、交換コードでセッションを確立する
  Future<void> login() async {
    if (kIsWeb) {
      // Web: ブラウザが Cookie を自動管理するため、単純なリダイレクトで十分
      final loginUrl = Uri.parse('$_baseUrl/auth/login');
      if (await canLaunchUrl(loginUrl)) {
        await launchUrl(loginUrl, mode: LaunchMode.externalApplication);
      }
    } else {
      // Mobile: flutter_web_auth_2 で OAuth フローを実行する
      await _loginMobile();
    }
  }

  /// モバイル向けの OAuth ログインフロー
  /// 1. flutter_web_auth_2 で BFF /auth/login を開く（カスタムスキームでリダイレクト）
  /// 2. 交換コードを取得する
  /// 3. /auth/exchange で交換コードをセッションに変換する
  Future<void> _loginMobile() async {
    final webAuthCallback = ref.read(webAuthCallbackProvider);
    if (webAuthCallback == null) {
      // POLY-008 監査対応: flutter_web_auth_2 が未設定の場合はサイレントフォールバックではなく
      // 明示的なエラーをスローする。設定不足のまま認証フローが不完全に進むことを防ぐ。
      // flutter_web_auth_2 を設定するには AuthClientProvider.overrideWithValue で
      // webAuthCallbackProvider をオーバーライドすること。
      throw StateError(
        '[k1s0-auth] flutter_web_auth_2 のコールバック関数が設定されていません。'
        'webAuthCallbackProvider を AuthClientProvider.overrideWithValue で設定してください。',
      );
    }

    final callbackScheme = ref.read(authCallbackSchemeProvider);
    final callbackUrl = '$callbackScheme://auth/callback';
    // FE-10 監査対応: redirect_to パラメータは URI エンコードが必要。
    // callbackUrl に "://" が含まれ BFF 側でパース時に切り捨てられる危険を防ぐ。
    final encodedCallbackUrl = Uri.encodeComponent(callbackUrl);
    final loginUrl = '$_baseUrl/auth/login?redirect_to=$encodedCallbackUrl';

    try {
      // flutter_web_auth_2 で OAuth フローを実行し、コールバック URL を取得する
      // ユーザーキャンセルやネットワークエラー時は例外が発生する
      final resultUrl = await webAuthCallback(
        url: loginUrl,
        callbackUrlScheme: callbackScheme,
      );

      // コールバック URL から交換コードを抽出する
      final uri = Uri.parse(resultUrl);
      final code = uri.queryParameters['code'];
      if (code == null) return;

      // BFF に交換コードを送信してセッションクッキーを取得する
      final response = await _apiClient.get<Map<String, dynamic>>(
        '/auth/exchange',
        queryParameters: {'code': code},
      );
      final data = response.data;
      if (data != null && data['authenticated'] == true) {
        _csrfToken = data['csrf_token'] as String?;
        state = AuthAuthenticated(userId: data['id'] as String);
      }
    } catch (e) {
      // ユーザーキャンセル・ネットワークエラー・交換失敗時は未認証状態を維持する
      state = const AuthUnauthenticated();
    }
  }

  /// BFF のログアウトエンドポイントを呼び出してセッションを破棄する。
  /// ネットワークエラーが発生してもクライアント側のセッション情報は必ず削除する（finally で保証）。
  Future<void> logout() async {
    try {
      await _apiClient.post<void>('/auth/logout');
    } catch (_) {
      // サーバー側のログアウトが失敗してもクライアント側のセッションは必ずクリアする
    } finally {
      _csrfToken = null;
      if (!kIsWeb) {
        // clearSession() は非同期メソッドのため await が必須
        await _sessionCookieInterceptor.clearSession();
      }
      state = const AuthUnauthenticated();
    }
  }
}

final authProvider = NotifierProvider<AuthNotifier, AuthState>(
  AuthNotifier.new,
);
