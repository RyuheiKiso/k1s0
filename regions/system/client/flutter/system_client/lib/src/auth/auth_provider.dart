import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'auth_state.dart';
import '../http/api_client.dart';

/// BFF API ベース URL を提供する Provider
final authApiBaseUrlProvider = Provider<String>(
  (_) => '/bff',
);

class AuthNotifier extends Notifier<AuthState> {
  late final Dio _apiClient;

  @override
  AuthState build() {
    final baseUrl = ref.read(authApiBaseUrlProvider);
    _apiClient = ApiClient.create(baseUrl: baseUrl);
    _checkSession();
    return const AuthUnauthenticated();
  }

  /// セッション確認
  Future<void> _checkSession() async {
    try {
      final response = await _apiClient.get<Map<String, dynamic>>('/auth/me');
      final data = response.data;
      if (data != null && data['id'] != null) {
        state = AuthAuthenticated(userId: data['id'] as String);
      }
    } catch (_) {
      state = const AuthUnauthenticated();
    }
  }

  Future<void> login({
    required String username,
    required String password,
  }) async {
    final response = await _apiClient.post<Map<String, dynamic>>(
      '/auth/login',
      data: {'username': username, 'password': password},
    );
    final data = response.data;
    if (data != null && data['id'] != null) {
      state = AuthAuthenticated(userId: data['id'] as String);
    }
  }

  Future<void> logout() async {
    await _apiClient.post<void>('/auth/logout');
    state = const AuthUnauthenticated();
  }
}

final authProvider = NotifierProvider<AuthNotifier, AuthState>(
  AuthNotifier.new,
);
