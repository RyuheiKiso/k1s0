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
  final isOnLoginPage = location == loginPath;

  if (!isAuthenticated && !isOnLoginPage) {
    return loginPath;
  }

  return null;
}
