sealed class AuthState {
  const AuthState();
}

class AuthUnauthenticated extends AuthState {
  const AuthUnauthenticated();

  @override
  bool operator ==(Object other) => other is AuthUnauthenticated;

  @override
  int get hashCode => runtimeType.hashCode;
}

class AuthAuthenticated extends AuthState {
  const AuthAuthenticated({required this.userId});

  final String userId;

  @override
  bool operator ==(Object other) =>
      other is AuthAuthenticated && other.userId == userId;

  @override
  int get hashCode => userId.hashCode;
}
