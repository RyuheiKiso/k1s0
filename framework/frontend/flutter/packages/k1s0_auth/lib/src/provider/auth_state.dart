import 'package:freezed_annotation/freezed_annotation.dart';

import '../types/auth_error.dart';
import '../types/auth_user.dart';

part 'auth_state.freezed.dart';

/// Authentication status
enum AuthStatus {
  /// Initial state, checking stored tokens
  initial,

  /// Loading/refreshing authentication
  loading,

  /// Authenticated
  authenticated,

  /// Not authenticated
  unauthenticated,

  /// Authentication error
  error,
}

/// Authentication state
@freezed
class AuthState with _$AuthState {
  /// Creates an authentication state
  const factory AuthState({
    /// Current status
    @Default(AuthStatus.initial) AuthStatus status,

    /// Authenticated user (when authenticated)
    AuthUser? user,

    /// Error information (when error)
    AuthError? error,
  }) = _AuthState;

  const AuthState._();

  /// Initial state
  static const initial = AuthState();

  /// Loading state
  static const loading = AuthState(status: AuthStatus.loading);

  /// Unauthenticated state
  static const unauthenticated = AuthState(status: AuthStatus.unauthenticated);

  /// Create authenticated state
  factory AuthState.authenticated(AuthUser user) => AuthState(
        status: AuthStatus.authenticated,
        user: user,
      );

  /// Create error state
  factory AuthState.failure(AuthError error) => AuthState(
        status: AuthStatus.error,
        error: error,
      );

  /// Whether currently loading
  bool get isLoading => status == AuthStatus.loading;

  /// Whether authenticated
  bool get isAuthenticated => status == AuthStatus.authenticated;

  /// Whether unauthenticated
  bool get isUnauthenticated => status == AuthStatus.unauthenticated;

  /// Whether there's an error
  bool get hasError => status == AuthStatus.error;

  /// Whether the initial check is complete
  bool get isInitialized => status != AuthStatus.initial;
}
