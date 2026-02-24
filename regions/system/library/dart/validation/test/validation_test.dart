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

    test('has code field', () {
      const err = ValidationError('email', 'bad', code: 'INVALID_EMAIL');
      expect(err.code, equals('INVALID_EMAIL'));
    });

    test('default code is derived from field', () {
      const err = ValidationError('email', 'bad');
      expect(err.code, equals('INVALID_EMAIL'));
    });
  });

  group('validatePagination', () {
    test('accepts valid pagination', () {
      expect(() => validatePagination(1, 10), returnsNormally);
      expect(() => validatePagination(1, 1), returnsNormally);
      expect(() => validatePagination(1, 100), returnsNormally);
      expect(() => validatePagination(999, 50), returnsNormally);
    });

    test('rejects page < 1', () {
      expect(() => validatePagination(0, 10), throwsA(isA<ValidationError>()));
      expect(() => validatePagination(-1, 10), throwsA(isA<ValidationError>()));
    });

    test('rejects perPage out of range', () {
      expect(() => validatePagination(1, 0), throwsA(isA<ValidationError>()));
      expect(() => validatePagination(1, 101), throwsA(isA<ValidationError>()));
    });

    test('error has correct code', () {
      try {
        validatePagination(0, 10);
      } on ValidationError catch (e) {
        expect(e.code, equals('INVALID_PAGE'));
      }
      try {
        validatePagination(1, 0);
      } on ValidationError catch (e) {
        expect(e.code, equals('INVALID_PER_PAGE'));
      }
    });
  });

  group('validateDateRange', () {
    test('accepts valid date range', () {
      final start = DateTime(2024, 1, 1);
      final end = DateTime(2024, 12, 31);
      expect(() => validateDateRange(start, end), returnsNormally);
    });

    test('accepts equal dates', () {
      final dt = DateTime(2024, 6, 15);
      expect(() => validateDateRange(dt, dt), returnsNormally);
    });

    test('rejects start after end', () {
      final start = DateTime(2024, 12, 31);
      final end = DateTime(2024, 1, 1);
      expect(() => validateDateRange(start, end), throwsA(isA<ValidationError>()));
    });

    test('error has correct code', () {
      try {
        validateDateRange(DateTime(2024, 12, 31), DateTime(2024, 1, 1));
      } on ValidationError catch (e) {
        expect(e.code, equals('INVALID_DATE_RANGE'));
      }
    });
  });

  group('ValidationErrors', () {
    test('empty collection has no errors', () {
      final errors = ValidationErrors();
      expect(errors.hasErrors(), isFalse);
      expect(errors.getErrors(), isEmpty);
    });

    test('adding errors works', () {
      final errors = ValidationErrors();
      errors.add(const ValidationError('email', 'bad', code: 'INVALID_EMAIL'));
      errors.add(const ValidationError('page', 'bad', code: 'INVALID_PAGE'));

      expect(errors.hasErrors(), isTrue);
      expect(errors.getErrors(), hasLength(2));
      expect(errors.getErrors()[0].code, equals('INVALID_EMAIL'));
      expect(errors.getErrors()[1].code, equals('INVALID_PAGE'));
    });
  });
}
