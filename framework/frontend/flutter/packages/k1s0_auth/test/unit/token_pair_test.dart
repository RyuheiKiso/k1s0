import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_auth/src/token/token_pair.dart';

void main() {
  group('TokenPair', () {
    test('creates with required fields', () {
      const pair = TokenPair(accessToken: 'access-token-123');

      expect(pair.accessToken, 'access-token-123');
      expect(pair.refreshToken, isNull);
      expect(pair.idToken, isNull);
      expect(pair.expiresAt, isNull);
      expect(pair.tokenType, 'Bearer');
      expect(pair.scope, isNull);
    });

    test('creates with all fields', () {
      const pair = TokenPair(
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        idToken: 'id-token',
        expiresAt: 1700000000000,
        tokenType: 'Bearer',
        scope: 'openid profile',
      );

      expect(pair.accessToken, 'access-token');
      expect(pair.refreshToken, 'refresh-token');
      expect(pair.idToken, 'id-token');
      expect(pair.expiresAt, 1700000000000);
      expect(pair.tokenType, 'Bearer');
      expect(pair.scope, 'openid profile');
    });

    test('fromJson creates correct instance', () {
      final json = {
        'access_token': 'access-from-json',
        'refresh_token': 'refresh-from-json',
        'id_token': 'id-from-json',
        'expires_at': 1700000000000,
        'token_type': 'Bearer',
        'scope': 'openid',
      };

      final pair = TokenPair.fromJson(json);

      expect(pair.accessToken, 'access-from-json');
      expect(pair.refreshToken, 'refresh-from-json');
      expect(pair.idToken, 'id-from-json');
      expect(pair.expiresAt, 1700000000000);
    });

    test('toJson returns correct map', () {
      const pair = TokenPair(
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
      );

      final json = pair.toJson();

      expect(json['access_token'], 'access-token');
      expect(json['refresh_token'], 'refresh-token');
    });

    group('isExpired', () {
      test('returns false when expiresAt is null', () {
        const pair = TokenPair(accessToken: 'token');

        expect(pair.isExpired, false);
      });

      test('returns true when token is expired', () {
        // Expired 1 hour ago
        final expiresAt =
            DateTime.now().millisecondsSinceEpoch - const Duration(hours: 1).inMilliseconds;
        final pair = TokenPair(accessToken: 'token', expiresAt: expiresAt);

        expect(pair.isExpired, true);
      });

      test('returns false when token is not expired', () {
        // Expires in 1 hour
        final expiresAt =
            DateTime.now().millisecondsSinceEpoch + const Duration(hours: 1).inMilliseconds;
        final pair = TokenPair(accessToken: 'token', expiresAt: expiresAt);

        expect(pair.isExpired, false);
      });
    });

    group('willExpireIn', () {
      test('returns false when expiresAt is null', () {
        const pair = TokenPair(accessToken: 'token');

        expect(pair.willExpireIn(const Duration(minutes: 5)), false);
      });

      test('returns true when token will expire within duration', () {
        // Expires in 3 minutes
        final expiresAt =
            DateTime.now().millisecondsSinceEpoch + const Duration(minutes: 3).inMilliseconds;
        final pair = TokenPair(accessToken: 'token', expiresAt: expiresAt);

        expect(pair.willExpireIn(const Duration(minutes: 5)), true);
      });

      test('returns false when token will not expire within duration', () {
        // Expires in 10 minutes
        final expiresAt =
            DateTime.now().millisecondsSinceEpoch + const Duration(minutes: 10).inMilliseconds;
        final pair = TokenPair(accessToken: 'token', expiresAt: expiresAt);

        expect(pair.willExpireIn(const Duration(minutes: 5)), false);
      });
    });

    test('expirationTime returns correct DateTime', () {
      const expiresAt = 1700000000000;
      const pair = TokenPair(accessToken: 'token', expiresAt: expiresAt);

      expect(pair.expirationTime,
          DateTime.fromMillisecondsSinceEpoch(expiresAt));
    });

    test('expirationTime returns null when expiresAt is null', () {
      const pair = TokenPair(accessToken: 'token');

      expect(pair.expirationTime, isNull);
    });

    group('hasRefreshToken', () {
      test('returns true when refresh token exists', () {
        const pair = TokenPair(
          accessToken: 'token',
          refreshToken: 'refresh',
        );

        expect(pair.hasRefreshToken, true);
      });

      test('returns false when refresh token is null', () {
        const pair = TokenPair(accessToken: 'token');

        expect(pair.hasRefreshToken, false);
      });

      test('returns false when refresh token is empty', () {
        const pair = TokenPair(
          accessToken: 'token',
          refreshToken: '',
        );

        expect(pair.hasRefreshToken, false);
      });
    });

    test('authorizationHeader returns correct format', () {
      const pair = TokenPair(
        accessToken: 'my-access-token',
        tokenType: 'Bearer',
      );

      expect(pair.authorizationHeader, 'Bearer my-access-token');
    });
  });
}
