import '../auth/auth_state.dart';

/// GoRouter の redirect 関数で使用するガードロジック。
///
/// 未認証の場合は [loginPath] を返し、認証済みまたは
/// すでに [loginPath] にいる場合は null を返す。
String? authGuardRedirect({
  required AuthState authState,
  required String location,
  String loginPath = '/login',
}) {
  final isAuthenticated = authState is AuthAuthenticated;

  // FE-11 対応: location の単純な等値比較ではクエリパラメータ付きの URL（例: /login?redirect=/home）や
  // サブパス（例: /login/sso）を同一ページとして認識できずリダイレクトループが発生する。
  // loginPath 自身・クエリパラメータ付き・サブパスの3パターンをすべてログインページとして扱う。
  final isOnLoginPage = location == loginPath ||
      location.startsWith('$loginPath?') ||
      location.startsWith('$loginPath/');

  if (!isAuthenticated && !isOnLoginPage) {
    return loginPath;
  }

  return null;
}
