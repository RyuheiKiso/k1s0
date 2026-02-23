import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:system_client/system_client.dart';

void main() {
  group('AuthGuard', () {
    test('未認証時は /login にリダイレクトする', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      // 未認証状態での redirect は "/login" を返す
      final redirect = authGuardRedirect(
        authState: const AuthUnauthenticated(),
        location: '/dashboard',
        loginPath: '/login',
      );

      expect(redirect, equals('/login'));
    });

    test('認証済みの場合は null を返す（リダイレクトなし）', () {
      final redirect = authGuardRedirect(
        authState: const AuthAuthenticated(userId: 'user-123'),
        location: '/dashboard',
        loginPath: '/login',
      );

      expect(redirect, isNull);
    });

    test('未認証で /login にアクセスした場合は null を返す', () {
      final redirect = authGuardRedirect(
        authState: const AuthUnauthenticated(),
        location: '/login',
        loginPath: '/login',
      );

      expect(redirect, isNull);
    });
  });
}
