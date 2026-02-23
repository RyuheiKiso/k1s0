import 'package:test/test.dart';

import 'package:k1s0_validation/validation.dart';

void main() {
  group('validateEmail', () {
    test('accepts valid email', () {
      expect(() => validateEmail('user@example.com'), returnsNormally);
    });

    test('accepts email with dots', () {
      expect(() => validateEmail('first.last@example.co.jp'), returnsNormally);
    });

    test('rejects email without @', () {
      expect(() => validateEmail('invalid'), throwsA(isA<ValidationError>()));
    });

    test('rejects email without domain', () {
      expect(() => validateEmail('user@'), throwsA(isA<ValidationError>()));
    });
  });

  group('validateUuid', () {
    test('accepts valid UUID v4', () {
      expect(() => validateUuid('550e8400-e29b-41d4-a716-446655440000'), returnsNormally);
    });

    test('rejects invalid UUID', () {
      expect(() => validateUuid('not-a-uuid'), throwsA(isA<ValidationError>()));
    });

    test('rejects UUID v1 format', () {
      expect(() => validateUuid('550e8400-e29b-11d4-a716-446655440000'), throwsA(isA<ValidationError>()));
    });
  });

  group('validateUrl', () {
    test('accepts https URL', () {
      expect(() => validateUrl('https://example.com'), returnsNormally);
    });

    test('accepts http URL', () {
      expect(() => validateUrl('http://example.com/path'), returnsNormally);
    });

    test('rejects URL without scheme', () {
      expect(() => validateUrl('example.com'), throwsA(isA<ValidationError>()));
    });

    test('rejects ftp URL', () {
      expect(() => validateUrl('ftp://example.com'), throwsA(isA<ValidationError>()));
    });
  });

  group('validateTenantId', () {
    test('accepts valid tenant ID', () {
      expect(() => validateTenantId('my-tenant-1'), returnsNormally);
    });

    test('rejects too short', () {
      expect(() => validateTenantId('ab'), throwsA(isA<ValidationError>()));
    });

    test('rejects uppercase', () {
      expect(() => validateTenantId('MyTenant'), throwsA(isA<ValidationError>()));
    });

    test('rejects special characters', () {
      expect(() => validateTenantId('my_tenant!'), throwsA(isA<ValidationError>()));
    });
  });

  group('ValidationError', () {
    test('has correct fields and toString', () {
      const err = ValidationError('email', 'bad');
      expect(err.field, equals('email'));
      expect(err.message, equals('bad'));
      expect(err.toString(), contains('email'));
    });
  });
}
