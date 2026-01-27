/// Authentication error codes
enum AuthErrorCode {
  /// Invalid token format or structure
  invalidToken,

  /// Token has expired
  tokenExpired,

  /// Token refresh failed
  refreshFailed,

  /// Network error during authentication
  networkError,

  /// OIDC-specific error
  oidcError,

  /// Unauthorized (no valid credentials)
  unauthorized,

  /// Forbidden (valid credentials but no permission)
  forbidden,

  /// Unknown error
  unknown,
}

/// Authentication error
class AuthError implements Exception {
  /// Creates an authentication error
  AuthError({
    required this.code,
    required this.message,
    this.cause,
    this.stackTrace,
  });

  /// Error code
  final AuthErrorCode code;

  /// Error message
  final String message;

  /// Original exception
  final Object? cause;

  /// Stack trace
  final StackTrace? stackTrace;

  @override
  String toString() => 'AuthError[$code]: $message';

  /// Whether this error is recoverable by re-authentication
  bool get isRecoverable {
    switch (code) {
      case AuthErrorCode.tokenExpired:
      case AuthErrorCode.unauthorized:
        return true;
      case AuthErrorCode.invalidToken:
      case AuthErrorCode.refreshFailed:
      case AuthErrorCode.networkError:
      case AuthErrorCode.oidcError:
      case AuthErrorCode.forbidden:
      case AuthErrorCode.unknown:
        return false;
    }
  }
}

/// Extension methods for AuthError
extension AuthErrorExtension on AuthError {
  /// Get a user-friendly error message in Japanese
  String get localizedMessage {
    switch (code) {
      case AuthErrorCode.invalidToken:
        return '認証情報が無効です。再度ログインしてください。';
      case AuthErrorCode.tokenExpired:
        return 'セッションの有効期限が切れました。再度ログインしてください。';
      case AuthErrorCode.refreshFailed:
        return '認証の更新に失敗しました。再度ログインしてください。';
      case AuthErrorCode.networkError:
        return 'ネットワークエラーが発生しました。接続を確認してください。';
      case AuthErrorCode.oidcError:
        return '認証サーバーでエラーが発生しました。';
      case AuthErrorCode.unauthorized:
        return '認証が必要です。ログインしてください。';
      case AuthErrorCode.forbidden:
        return 'この操作を行う権限がありません。';
      case AuthErrorCode.unknown:
        return '認証エラーが発生しました。';
    }
  }
}
