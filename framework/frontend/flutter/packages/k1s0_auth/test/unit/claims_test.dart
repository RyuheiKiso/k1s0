import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_auth/src/token/claims.dart';

void main() {
  group('Claims', () {
    test('creates with required fields', () {
      const claims = Claims(
        sub: 'user-123',
        iss: 'https://auth.example.com',
        exp: 1700000000,
        iat: 1699996400,
      );

      expect(claims.sub, 'user-123');
      expect(claims.iss, 'https://auth.example.com');
      expect(claims.exp, 1700000000);
      expect(claims.iat, 1699996400);
      expect(claims.aud, isNull);
      expect(claims.nbf, isNull);
      expect(claims.jti, isNull);
      expect(claims.roles, isEmpty);
      expect(claims.permissions, isEmpty);
      expect(claims.tenantId, isNull);
      expect(claims.scope, isNull);
    });

    test('creates with all fields', () {
      const claims = Claims(
        sub: 'user-456',
        iss: 'https://auth.example.com',
        exp: 1700000000,
        iat: 1699996400,
        aud: ['client-123', 'api'],
        nbf: 1699996400,
        jti: 'token-id-789',
        roles: ['admin', 'user'],
        permissions: ['read', 'write'],
        tenantId: 'tenant-abc',
        scope: 'openid profile',
      );

      expect(claims.aud, ['client-123', 'api']);
      expect(claims.nbf, 1699996400);
      expect(claims.jti, 'token-id-789');
      expect(claims.roles, ['admin', 'user']);
      expect(claims.permissions, ['read', 'write']);
      expect(claims.tenantId, 'tenant-abc');
      expect(claims.scope, 'openid profile');
    });

    test('fromJson creates correct instance', () {
      final json = {
        'sub': 'user-json',
        'iss': 'https://auth.example.com',
        'exp': 1700000000,
        'iat': 1699996400,
        'roles': ['admin'],
        'permissions': ['read'],
        'tenant_id': 'tenant-json',
      };

      final claims = Claims.fromJson(json);

      expect(claims.sub, 'user-json');
      expect(claims.roles, ['admin']);
      expect(claims.permissions, ['read']);
      expect(claims.tenantId, 'tenant-json');
    });

    test('fromJson handles string audience', () {
      final json = {
        'sub': 'user',
        'iss': 'https://auth.example.com',
        'exp': 1700000000,
        'iat': 1699996400,
        'aud': 'single-audience',
      };

      final claims = Claims.fromJson(json);

      expect(claims.aud, ['single-audience']);
    });

    test('fromJson handles list audience', () {
      final json = {
        'sub': 'user',
        'iss': 'https://auth.example.com',
        'exp': 1700000000,
        'iat': 1699996400,
        'aud': ['aud1', 'aud2'],
      };

      final claims = Claims.fromJson(json);

      expect(claims.aud, ['aud1', 'aud2']);
    });

    group('isExpired', () {
      test('returns true when token is expired', () {
        // Expired 1 hour ago
        final expiredAt =
            (DateTime.now().millisecondsSinceEpoch ~/ 1000) - const Duration(hours: 1).inSeconds;
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: expiredAt,
          iat: expiredAt - 3600,
        );

        expect(claims.isExpired, true);
      });

      test('returns false when token is not expired', () {
        // Expires in 1 hour
        final expiresAt =
            (DateTime.now().millisecondsSinceEpoch ~/ 1000) + const Duration(hours: 1).inSeconds;
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: expiresAt,
          iat: expiresAt - 3600,
        );

        expect(claims.isExpired, false);
      });
    });

    group('willExpireIn', () {
      test('returns true when token will expire within duration', () {
        // Expires in 3 minutes
        final expiresAt =
            (DateTime.now().millisecondsSinceEpoch ~/ 1000) + const Duration(minutes: 3).inSeconds;
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: expiresAt,
          iat: expiresAt - 3600,
        );

        expect(claims.willExpireIn(const Duration(minutes: 5)), true);
      });

      test('returns false when token will not expire within duration', () {
        // Expires in 10 minutes
        final expiresAt =
            (DateTime.now().millisecondsSinceEpoch ~/ 1000) + const Duration(minutes: 10).inSeconds;
        final claims = Claims(
          sub: 'user',
          iss: 'issuer',
          exp: expiresAt,
          iat: expiresAt - 3600,
        );

        expect(claims.willExpireIn(const Duration(minutes: 5)), false);
      });
    });

    test('expirationTime returns correct DateTime', () {
      const exp = 1700000000;
      const claims = Claims(
        sub: 'user',
        iss: 'issuer',
        exp: exp,
        iat: exp - 3600,
      );

      expect(claims.expirationTime, DateTime.fromMillisecondsSinceEpoch(exp * 1000));
    });

    test('issuedAtTime returns correct DateTime', () {
      const iat = 1699996400;
      const claims = Claims(
        sub: 'user',
        iss: 'issuer',
        exp: iat + 3600,
        iat: iat,
      );

      expect(claims.issuedAtTime, DateTime.fromMillisecondsSinceEpoch(iat * 1000));
    });

    group('role checks', () {
      const claims = Claims(
        sub: 'user',
        iss: 'issuer',
        exp: 1700000000,
        iat: 1699996400,
        roles: ['admin', 'editor'],
      );

      test('hasRole returns true for existing role', () {
        expect(claims.hasRole('admin'), true);
        expect(claims.hasRole('editor'), true);
      });

      test('hasRole returns false for non-existing role', () {
        expect(claims.hasRole('viewer'), false);
      });

      test('hasAnyRole returns true when any role matches', () {
        expect(claims.hasAnyRole(['viewer', 'admin']), true);
      });

      test('hasAnyRole returns false when no role matches', () {
        expect(claims.hasAnyRole(['viewer', 'guest']), false);
      });

      test('hasAllRoles returns true when all roles match', () {
        expect(claims.hasAllRoles(['admin', 'editor']), true);
      });

      test('hasAllRoles returns false when not all roles match', () {
        expect(claims.hasAllRoles(['admin', 'viewer']), false);
      });
    });

    group('permission checks', () {
      const claims = Claims(
        sub: 'user',
        iss: 'issuer',
        exp: 1700000000,
        iat: 1699996400,
        permissions: ['read', 'write'],
      );

      test('hasPermission returns true for existing permission', () {
        expect(claims.hasPermission('read'), true);
        expect(claims.hasPermission('write'), true);
      });

      test('hasPermission returns false for non-existing permission', () {
        expect(claims.hasPermission('delete'), false);
      });

      test('hasAnyPermission returns true when any permission matches', () {
        expect(claims.hasAnyPermission(['delete', 'read']), true);
      });

      test('hasAnyPermission returns false when no permission matches', () {
        expect(claims.hasAnyPermission(['delete', 'admin']), false);
      });

      test('hasAllPermissions returns true when all permissions match', () {
        expect(claims.hasAllPermissions(['read', 'write']), true);
      });

      test('hasAllPermissions returns false when not all permissions match', () {
        expect(claims.hasAllPermissions(['read', 'delete']), false);
      });
    });
  });
}
