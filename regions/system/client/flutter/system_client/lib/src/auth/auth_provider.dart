import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart' show kIsWeb, debugPrint;
// M-010 監査対応: PlatformException を使用してユーザーキャンセルを区別するためにインポートする
import 'package:flutter/services.dart' show PlatformException;
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

/// FE-005 監査対応: OAuth カスタムスキームをビルド時定数として設定可能にする
/// dart compile で --define=OAUTH_CALLBACK_SCHEME=myscheme を指定することで上書きできる。
/// 未指定時は 'k1s0' をデフォルト値として使用する。
const _oauthCallbackScheme = String.fromEnvironment(
  'OAUTH_CALLBACK_SCHEME',
  defaultValue: 'k1s0',
);

/// FE-005 監査対応: OAuth コールバックパスをビルド時定数として設定可能にする
/// dart compile で --define=OAUTH_CALLBACK_PATH=auth/callback を指定することで上書きできる。
/// 未指定時は 'auth/callback' をデフォルト値として使用する。
const _oauthCallbackPath = String.fromEnvironment(
  'OAUTH_CALLBACK_PATH',
  defaultValue: 'auth/callback',
);

/// モバイル OAuth のカスタム URL スキーム Provider
/// ビルド時定数 OAUTH_CALLBACK_SCHEME から取得したスキームを提供する。
/// テスト時は overrideWithValue で差し替え可能にする。
final authCallbackSchemeProvider = Provider<String>(
  (_) => _oauthCallbackScheme,
);

/// セッションクッキーインターセプター Provider（モバイル専用）
/// テスト時は FlutterSecureStorage を使わないモックインスタンスに差し替え可能にする
/// 本番では AuthNotifier.build() 内でセッション期限切れコールバック付きのインスタンスを生成する
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

    // テスト時はモック差し替え可能な Provider からインターセプターを取得する
    // ただし NoOpSessionCookieInterceptor 等のサブクラスが注入された場合はそのまま使用し、
    // デフォルト実装（SessionCookieInterceptor そのもの）の場合はセッション期限切れコールバックを付与して再生成する
    final injected = ref.read(sessionCookieInterceptorProvider);
    if (injected.runtimeType == SessionCookieInterceptor) {
      // 本番パス: onSessionExpired コールバックを注入してセッション有効期限管理を有効にする
      _sessionCookieInterceptor = SessionCookieInterceptor(
        cookieName: injected.cookieName,
        onSessionExpired: _handleSessionExpired,
      );
    } else {
      // テスト/モックパス: 注入されたインスタンスをそのまま使用する
      _sessionCookieInterceptor = injected;
    }

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

  /// セッション有効期限切れを検出した際にクライアント側のセッション情報をリセットする
  /// SessionCookieInterceptor から同期的に呼び出されるため、非同期処理は行わない
  void _handleSessionExpired() {
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
    } catch (e) {
      // 非同期操作完了後、プロバイダーが既に破棄されていれば何もしない
      if (!ref.mounted) return;
      // HIGH-019対応: 5xxエラーとネットワークエラーを区別する
      // 401/403はセッション失効として未認証状態に遷移する
      // 5xxサーバーエラーやネットワークエラーの場合は現在の認証状態を維持する
      if (e is DioException) {
        final statusCode = e.response?.statusCode;
        if (statusCode == 401 || statusCode == 403) {
          // セッション失効・認証エラーの場合は未認証状態に遷移する
          state = const AuthUnauthenticated();
        }
        // 5xxエラーやネットワークエラー（statusCode == null を含む）の場合は状態を維持する
        return;
      }
      // DioException 以外の予期しない例外は未認証状態として扱う
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
    // FE-005 監査対応: コールバックパスも定数 _oauthCallbackPath から取得してハードコードを排除する
    final callbackUrl = '$callbackScheme://$_oauthCallbackPath';
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
        // MED-009 監査対応: async 処理後に Provider が dispose されている場合、
        // state 更新で例外が発生するため ref.mounted チェックを追加する。
        if (!ref.mounted) return;
        state = AuthAuthenticated(userId: data['id'] as String);
      }
    // M-010 監査対応: エラーを分類してデバッグ情報を保持する
    on PlatformException catch (e) {
      // ユーザーキャンセルと認証失敗を区別する
      if (e.code == 'CANCELED') {
        debugPrint('User cancelled login');
        return;
      }
      debugPrint('Platform exception during login: ${e.message}');
      if (!ref.mounted) return;
      state = const AuthUnauthenticated();
      rethrow;
    } catch (e) {
      // ネットワークエラー・交換失敗時は未認証状態を維持する
      // MED-009 監査対応: async 処理後に Provider が dispose されている場合、
      // state 更新で例外が発生するため ref.mounted チェックを追加する。
      debugPrint('Login failed: $e');
      if (!ref.mounted) return;
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
      // MED-009 監査対応: async 処理後に Provider が dispose されている場合、
      // state 更新で例外が発生するため ref.mounted チェックを追加する。
      if (!ref.mounted) return;
      state = const AuthUnauthenticated();
    }
  }
}

final authProvider = NotifierProvider<AuthNotifier, AuthState>(
  AuthNotifier.new,
);
