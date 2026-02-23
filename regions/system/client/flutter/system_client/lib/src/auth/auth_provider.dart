import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'auth_state.dart';

class AuthNotifier extends Notifier<AuthState> {
  @override
  AuthState build() => const AuthUnauthenticated();

  Future<void> login({
    required String username,
    required String password,
  }) async {
    // 実際の実装では API 呼び出しを行う
    // ここではダミーの userId を設定
    state = AuthAuthenticated(userId: 'user-${username.hashCode}');
  }

  Future<void> logout() async {
    state = const AuthUnauthenticated();
  }
}

final authProvider = NotifierProvider<AuthNotifier, AuthState>(
  AuthNotifier.new,
);
