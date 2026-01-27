import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_auth/src/provider/auth_state.dart';
import 'package:k1s0_auth/src/token/claims.dart';
import 'package:k1s0_auth/src/types/auth_error.dart';
import 'package:k1s0_auth/src/types/auth_user.dart';

void main() {
  group('AuthStatus', () {
    test('contains all expected statuses', () {
      expect(AuthStatus.values, contains(AuthStatus.initial));
      expect(AuthStatus.values, contains(AuthStatus.loading));
      expect(AuthStatus.values, contains(AuthStatus.authenticated));
      expect(AuthStatus.values, contains(AuthStatus.unauthenticated));
      expect(AuthStatus.values, contains(AuthStatus.error));
    });
  });

  group('AuthState', () {
    test('default state has initial status', () {
      const state = AuthState();

      expect(state.status, AuthStatus.initial);
      expect(state.user, isNull);
      expect(state.error, isNull);
    });

    test('initial constant has correct values', () {
      expect(AuthState.initial.status, AuthStatus.initial);
      expect(AuthState.initial.user, isNull);
    });

    test('loading constant has correct values', () {
      expect(AuthState.loading.status, AuthStatus.loading);
      expect(AuthState.loading.user, isNull);
    });

    test('unauthenticated constant has correct values', () {
      expect(AuthState.unauthenticated.status, AuthStatus.unauthenticated);
      expect(AuthState.unauthenticated.user, isNull);
    });

    test('authenticated factory creates correct state', () {
      final claims = Claims(
        sub: 'user-123',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        roles: ['admin'],
      );
      final user = AuthUser.fromClaims(claims);

      final state = AuthState.authenticated(user);

      expect(state.status, AuthStatus.authenticated);
      expect(state.user, user);
      expect(state.error, isNull);
    });

    test('failure factory creates correct state', () {
      final error = AuthError(
        code: AuthErrorCode.tokenExpired,
        message: 'Token expired',
      );

      final state = AuthState.failure(error);

      expect(state.status, AuthStatus.error);
      expect(state.error, error);
      expect(state.user, isNull);
    });

    group('helper getters', () {
      test('isLoading returns true for loading status', () {
        expect(AuthState.loading.isLoading, true);
        expect(AuthState.initial.isLoading, false);
        expect(AuthState.unauthenticated.isLoading, false);
      });

      test('isAuthenticated returns true for authenticated status', () {
        final claims = Claims(
          sub: 'user-123',
          iss: 'issuer',
          exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
          iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        );
        final user = AuthUser.fromClaims(claims);
        final authenticated = AuthState.authenticated(user);

        expect(authenticated.isAuthenticated, true);
        expect(AuthState.initial.isAuthenticated, false);
        expect(AuthState.unauthenticated.isAuthenticated, false);
      });

      test('isUnauthenticated returns true for unauthenticated status', () {
        expect(AuthState.unauthenticated.isUnauthenticated, true);
        expect(AuthState.initial.isUnauthenticated, false);
        expect(AuthState.loading.isUnauthenticated, false);
      });

      test('hasError returns true for error status', () {
        final error = AuthError(
          code: AuthErrorCode.unknown,
          message: 'Error',
        );
        final errorState = AuthState.failure(error);

        expect(errorState.hasError, true);
        expect(AuthState.initial.hasError, false);
      });

      test('isInitialized returns true when not initial', () {
        expect(AuthState.initial.isInitialized, false);
        expect(AuthState.loading.isInitialized, true);
        expect(AuthState.unauthenticated.isInitialized, true);
      });
    });
  });

  group('AuthUser', () {
    test('fromClaims creates correct user', () {
      final claims = Claims(
        sub: 'user-456',
        iss: 'issuer',
        exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
        iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        roles: ['admin', 'editor'],
        permissions: ['read', 'write'],
        tenantId: 'tenant-abc',
      );

      final user = AuthUser.fromClaims(claims);

      expect(user.id, 'user-456');
      expect(user.roles, ['admin', 'editor']);
      expect(user.permissions, ['read', 'write']);
      expect(user.tenantId, 'tenant-abc');
      expect(user.claims, claims);
    });

    group('role checks', () {
      late AuthUser user;

      setUp(() {
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
          iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
          roles: ['admin', 'editor'],
        );
        user = AuthUser.fromClaims(claims);
      });

      test('hasRole returns true for existing role', () {
        expect(user.hasRole('admin'), true);
        expect(user.hasRole('editor'), true);
      });

      test('hasRole returns false for non-existing role', () {
        expect(user.hasRole('viewer'), false);
      });

      test('hasAnyRole returns true when any role matches', () {
        expect(user.hasAnyRole(['viewer', 'admin']), true);
      });

      test('hasAnyRole returns false when no role matches', () {
        expect(user.hasAnyRole(['viewer', 'guest']), false);
      });

      test('hasAllRoles returns true when all roles match', () {
        expect(user.hasAllRoles(['admin', 'editor']), true);
      });

      test('hasAllRoles returns false when not all roles match', () {
        expect(user.hasAllRoles(['admin', 'viewer']), false);
      });
    });

    group('permission checks', () {
      late AuthUser user;

      setUp(() {
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
          iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
          permissions: ['read', 'write'],
        );
        user = AuthUser.fromClaims(claims);
      });

      test('hasPermission returns true for existing permission', () {
        expect(user.hasPermission('read'), true);
        expect(user.hasPermission('write'), true);
      });

      test('hasPermission returns false for non-existing permission', () {
        expect(user.hasPermission('delete'), false);
      });

      test('hasAnyPermission returns true when any permission matches', () {
        expect(user.hasAnyPermission(['delete', 'read']), true);
      });

      test('hasAnyPermission returns false when no permission matches', () {
        expect(user.hasAnyPermission(['delete', 'admin']), false);
      });

      test('hasAllPermissions returns true when all permissions match', () {
        expect(user.hasAllPermissions(['read', 'write']), true);
      });

      test('hasAllPermissions returns false when not all permissions match', () {
        expect(user.hasAllPermissions(['read', 'delete']), false);
      });
    });

    group('isAdmin', () {
      test('returns true for admin role', () {
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
          iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
          roles: ['admin'],
        );
        final user = AuthUser.fromClaims(claims);

        expect(user.isAdmin, true);
      });

      test('returns true for administrator role', () {
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
          iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
          roles: ['administrator'],
        );
        final user = AuthUser.fromClaims(claims);

        expect(user.isAdmin, true);
      });

      test('returns false for non-admin user', () {
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: (DateTime.now().millisecondsSinceEpoch ~/ 1000) + 3600,
          iat: DateTime.now().millisecondsSinceEpoch ~/ 1000,
          roles: ['editor'],
        );
        final user = AuthUser.fromClaims(claims);

        expect(user.isAdmin, false);
      });
    });
  });
}
