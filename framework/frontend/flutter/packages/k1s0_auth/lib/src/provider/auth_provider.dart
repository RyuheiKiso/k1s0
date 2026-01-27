import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../storage/memory_token_storage.dart';
import '../storage/secure_token_storage.dart';
import '../storage/token_storage.dart';
import '../token/claims.dart';
import '../token/token_decoder.dart';
import '../token/token_manager.dart';
import '../token/token_pair.dart';
import '../types/auth_config.dart';
import '../types/auth_error.dart';
import '../types/auth_user.dart';
import 'auth_state.dart';

/// Authentication notifier for managing auth state
class AuthNotifier extends StateNotifier<AuthState> {
  /// Creates an auth notifier
  AuthNotifier({
    required TokenStorage storage,
    TokenRefresher? refresher,
    Duration refreshMargin = const Duration(minutes: 5),
    bool autoRefresh = true,
  }) : _tokenManager = TokenManager(
          storage: storage,
          refresher: refresher,
          refreshMargin: refreshMargin,
          autoRefresh: autoRefresh,
        ),
        super(AuthState.initial);

  final TokenManager _tokenManager;

  /// Initialize authentication by checking stored tokens
  Future<void> initialize() async {
    state = AuthState.loading;

    try {
      final result = await _tokenManager.getValidToken();

      switch (result) {
        case TokenValid(:final claims):
          state = AuthState.authenticated(AuthUser.fromClaims(claims));
        case TokenRefreshed(:final claims):
          state = AuthState.authenticated(AuthUser.fromClaims(claims));
        case TokenExpired():
          state = AuthState.unauthenticated;
        case TokenNone():
          state = AuthState.unauthenticated;
      }
    } catch (e) {
      state = AuthState.failure(
        AuthError(
          code: AuthErrorCode.unknown,
          message: 'Failed to initialize authentication: $e',
          cause: e,
        ),
      );
    }
  }

  /// Login with tokens
  Future<void> login(TokenPair tokens) async {
    state = AuthState.loading;

    try {
      // Decode and validate the token
      final claims = TokenDecoder.decode(tokens.accessToken);

      if (claims.isExpired) {
        state = AuthState.failure(
          AuthError(
            code: AuthErrorCode.tokenExpired,
            message: 'Token is expired',
          ),
        );
        return;
      }

      // Save tokens
      await _tokenManager.setTokens(tokens);

      state = AuthState.authenticated(AuthUser.fromClaims(claims));
    } on TokenDecodingError catch (e) {
      state = AuthState.failure(
        AuthError(
          code: AuthErrorCode.invalidToken,
          message: 'Invalid token format',
          cause: e,
        ),
      );
    } catch (e) {
      state = AuthState.failure(
        AuthError(
          code: AuthErrorCode.unknown,
          message: 'Login failed: $e',
          cause: e,
        ),
      );
    }
  }

  /// Logout
  Future<void> logout() async {
    await _tokenManager.clearTokens();
    state = AuthState.unauthenticated;
  }

  /// Refresh the token manually
  Future<void> refresh() async {
    final currentUser = state.user;
    state = AuthState.loading;

    try {
      final tokens = await _tokenManager.refreshNow();

      if (tokens == null) {
        state = AuthState.failure(
          AuthError(
            code: AuthErrorCode.refreshFailed,
            message: 'Token refresh failed',
          ),
        );
        return;
      }

      final claims = TokenDecoder.decode(tokens.accessToken);
      state = AuthState.authenticated(AuthUser.fromClaims(claims));
    } catch (e) {
      // Restore previous state if refresh fails
      if (currentUser != null) {
        state = AuthState.authenticated(currentUser);
      } else {
        state = AuthState.failure(
          AuthError(
            code: AuthErrorCode.refreshFailed,
            message: 'Token refresh failed: $e',
            cause: e,
          ),
        );
      }
    }
  }

  /// Get the current access token
  Future<String?> getAccessToken() async {
    final result = await _tokenManager.getValidToken();
    switch (result) {
      case TokenValid(:final token):
        return token;
      case TokenRefreshed(:final token):
        return token;
      default:
        return null;
    }
  }

  /// Dispose resources
  @override
  void dispose() {
    _tokenManager.dispose();
    super.dispose();
  }
}

/// Provider for token storage
final tokenStorageProvider = Provider<TokenStorage>((ref) {
  return SecureTokenStorage();
});

/// Provider for auth configuration
final authConfigProvider = Provider<AuthConfig>((ref) {
  return const AuthConfig();
});

/// Provider for token refresher
final tokenRefresherProvider = Provider<TokenRefresher?>((ref) {
  // Override this provider to provide a token refresher
  return null;
});

/// Main authentication provider
final authProvider = StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  final storage = ref.watch(tokenStorageProvider);
  final refresher = ref.watch(tokenRefresherProvider);
  final config = ref.watch(authConfigProvider);

  final notifier = AuthNotifier(
    storage: storage,
    refresher: refresher,
    refreshMargin: Duration(seconds: config.refreshMarginSeconds),
    autoRefresh: config.autoRefresh,
  );

  // Initialize on creation
  notifier.initialize();

  return notifier;
});

/// Provider for the current user
final currentUserProvider = Provider<AuthUser?>((ref) {
  return ref.watch(authProvider).user;
});

/// Provider to check if authenticated
final isAuthenticatedProvider = Provider<bool>((ref) {
  return ref.watch(authProvider).isAuthenticated;
});

/// Provider to check if a user has a specific role
final hasRoleProvider = Provider.family<bool, String>((ref, role) {
  return ref.watch(currentUserProvider)?.hasRole(role) ?? false;
});

/// Provider to check if a user has any of the specified roles
final hasAnyRoleProvider = Provider.family<bool, List<String>>((ref, roles) {
  return ref.watch(currentUserProvider)?.hasAnyRole(roles) ?? false;
});

/// Provider to check if a user has a specific permission
final hasPermissionProvider = Provider.family<bool, String>((ref, permission) {
  return ref.watch(currentUserProvider)?.hasPermission(permission) ?? false;
});

/// Provider to check if a user has any of the specified permissions
final hasAnyPermissionProvider =
    Provider.family<bool, List<String>>((ref, permissions) {
  return ref.watch(currentUserProvider)?.hasAnyPermission(permissions) ?? false;
});

/// Create a custom auth provider with specific configuration
AuthNotifier createAuthNotifier({
  TokenStorage? storage,
  TokenRefresher? refresher,
  Duration refreshMargin = const Duration(minutes: 5),
  bool autoRefresh = true,
}) {
  return AuthNotifier(
    storage: storage ?? MemoryTokenStorage(),
    refresher: refresher,
    refreshMargin: refreshMargin,
    autoRefresh: autoRefresh,
  );
}
