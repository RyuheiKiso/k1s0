# k1s0_auth (Flutter)

← [Flutter パッケージ一覧](./)

## 目的

JWT/OIDC 認証クライアント。トークン管理、認証状態管理、認証ガード、GoRouter 統合を提供。

## 主要な型

```dart
@freezed
class Claims with _$Claims {
  const factory Claims({
    required String sub,
    required String iss,
    String? aud,
    required int exp,
    required int iat,
    @Default([]) List<String> roles,
    @Default([]) List<String> permissions,
    String? tenantId,
  }) = _Claims;
}

@freezed
class AuthState with _$AuthState {
  const factory AuthState.initial() = AuthInitial;
  const factory AuthState.loading() = AuthLoading;
  const factory AuthState.authenticated(AuthUser user) = AuthAuthenticated;
  const factory AuthState.unauthenticated() = AuthUnauthenticated;
  const factory AuthState.error(AuthError error) = AuthError;
}

class AuthNotifier extends StateNotifier<AuthState> {
  Future<void> login(String accessToken, {String? refreshToken});
  Future<void> logout();
  Future<void> refreshTokens();
}
```

## 使用例

```dart
// AuthProvider で認証状態を管理
final authState = ref.watch(authProvider);

authState.when(
  initial: () => SplashScreen(),
  loading: () => LoadingScreen(),
  authenticated: (user) => HomePage(),
  unauthenticated: () => LoginPage(),
  error: (error) => ErrorPage(error),
);

// 認証ガード
AuthGuard(
  child: DashboardPage(),
  unauthenticatedBuilder: (context) => LoginPage(),
)

// ロールベースの認可
RequireRole(
  roles: ['admin'],
  child: AdminPanel(),
  fallback: AccessDenied(),
)

// GoRouter 統合
final router = GoRouter(
  redirect: authGuard(
    ref,
    redirectTo: '/login',
    allowedPaths: ['/login', '/register'],
  ),
  routes: [...],
);
```
