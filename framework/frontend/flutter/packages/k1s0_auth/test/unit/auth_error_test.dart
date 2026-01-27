import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_auth/src/types/auth_error.dart';

void main() {
  group('AuthErrorCode', () {
    test('contains all expected codes', () {
      expect(AuthErrorCode.values, contains(AuthErrorCode.invalidToken));
      expect(AuthErrorCode.values, contains(AuthErrorCode.tokenExpired));
      expect(AuthErrorCode.values, contains(AuthErrorCode.refreshFailed));
      expect(AuthErrorCode.values, contains(AuthErrorCode.networkError));
      expect(AuthErrorCode.values, contains(AuthErrorCode.oidcError));
      expect(AuthErrorCode.values, contains(AuthErrorCode.unauthorized));
      expect(AuthErrorCode.values, contains(AuthErrorCode.forbidden));
      expect(AuthErrorCode.values, contains(AuthErrorCode.unknown));
    });
  });

  group('AuthError', () {
    test('creates with required fields', () {
      final error = AuthError(
        code: AuthErrorCode.tokenExpired,
        message: 'Token has expired',
      );

      expect(error.code, AuthErrorCode.tokenExpired);
      expect(error.message, 'Token has expired');
      expect(error.cause, isNull);
      expect(error.stackTrace, isNull);
    });

    test('creates with all fields', () {
      final cause = Exception('Original error');
      final stackTrace = StackTrace.current;

      final error = AuthError(
        code: AuthErrorCode.networkError,
        message: 'Network failure',
        cause: cause,
        stackTrace: stackTrace,
      );

      expect(error.code, AuthErrorCode.networkError);
      expect(error.message, 'Network failure');
      expect(error.cause, cause);
      expect(error.stackTrace, stackTrace);
    });

    test('toString includes code and message', () {
      final error = AuthError(
        code: AuthErrorCode.unauthorized,
        message: 'Not authenticated',
      );

      final str = error.toString();

      expect(str, contains('AuthError'));
      expect(str, contains('unauthorized'));
      expect(str, contains('Not authenticated'));
    });

    group('isRecoverable', () {
      test('returns true for tokenExpired', () {
        final error = AuthError(
          code: AuthErrorCode.tokenExpired,
          message: 'Token expired',
        );

        expect(error.isRecoverable, true);
      });

      test('returns true for unauthorized', () {
        final error = AuthError(
          code: AuthErrorCode.unauthorized,
          message: 'Unauthorized',
        );

        expect(error.isRecoverable, true);
      });

      test('returns false for invalidToken', () {
        final error = AuthError(
          code: AuthErrorCode.invalidToken,
          message: 'Invalid token',
        );

        expect(error.isRecoverable, false);
      });

      test('returns false for refreshFailed', () {
        final error = AuthError(
          code: AuthErrorCode.refreshFailed,
          message: 'Refresh failed',
        );

        expect(error.isRecoverable, false);
      });

      test('returns false for networkError', () {
        final error = AuthError(
          code: AuthErrorCode.networkError,
          message: 'Network error',
        );

        expect(error.isRecoverable, false);
      });

      test('returns false for forbidden', () {
        final error = AuthError(
          code: AuthErrorCode.forbidden,
          message: 'Forbidden',
        );

        expect(error.isRecoverable, false);
      });

      test('returns false for oidcError', () {
        final error = AuthError(
          code: AuthErrorCode.oidcError,
          message: 'OIDC error',
        );

        expect(error.isRecoverable, false);
      });

      test('returns false for unknown', () {
        final error = AuthError(
          code: AuthErrorCode.unknown,
          message: 'Unknown error',
        );

        expect(error.isRecoverable, false);
      });
    });
  });

  group('AuthErrorExtension', () {
    test('localizedMessage returns Japanese for invalidToken', () {
      final error = AuthError(
        code: AuthErrorCode.invalidToken,
        message: '',
      );

      expect(error.localizedMessage, contains('認証'));
    });

    test('localizedMessage returns Japanese for tokenExpired', () {
      final error = AuthError(
        code: AuthErrorCode.tokenExpired,
        message: '',
      );

      expect(error.localizedMessage, contains('セッション'));
    });

    test('localizedMessage returns Japanese for networkError', () {
      final error = AuthError(
        code: AuthErrorCode.networkError,
        message: '',
      );

      expect(error.localizedMessage, contains('ネットワーク'));
    });

    test('localizedMessage returns Japanese for forbidden', () {
      final error = AuthError(
        code: AuthErrorCode.forbidden,
        message: '',
      );

      expect(error.localizedMessage, contains('権限'));
    });

    test('all error codes have localized messages', () {
      for (final code in AuthErrorCode.values) {
        final error = AuthError(code: code, message: '');
        expect(error.localizedMessage.isNotEmpty, true,
            reason: 'Missing localized message for $code');
      }
    });
  });
}
