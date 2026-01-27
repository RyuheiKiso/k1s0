import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_ui/src/form/validation.dart';

void main() {
  group('K1s0Validators', () {
    group('required', () {
      test('returns error for null value', () {
        final result = K1s0Validators.required(null);

        expect(result, isNotNull);
        expect(result, contains('必須'));
      });

      test('returns error for empty string', () {
        final result = K1s0Validators.required('');

        expect(result, isNotNull);
      });

      test('returns error for whitespace only', () {
        final result = K1s0Validators.required('   ');

        expect(result, isNotNull);
      });

      test('returns null for valid value', () {
        final result = K1s0Validators.required('value');

        expect(result, isNull);
      });

      test('includes field name in error message', () {
        final result = K1s0Validators.required(null, 'メール');

        expect(result, contains('メール'));
      });
    });

    group('email', () {
      test('returns null for empty value (use required for that)', () {
        expect(K1s0Validators.email(null), isNull);
        expect(K1s0Validators.email(''), isNull);
      });

      test('returns error for invalid email', () {
        expect(K1s0Validators.email('invalid'), isNotNull);
        expect(K1s0Validators.email('invalid@'), isNotNull);
        expect(K1s0Validators.email('@example.com'), isNotNull);
        expect(K1s0Validators.email('invalid@example'), isNotNull);
      });

      test('returns null for valid email', () {
        expect(K1s0Validators.email('test@example.com'), isNull);
        expect(K1s0Validators.email('user.name@example.co.jp'), isNull);
        expect(K1s0Validators.email('user+tag@example.com'), isNull);
      });
    });

    group('phone', () {
      test('returns null for empty value', () {
        expect(K1s0Validators.phone(null), isNull);
        expect(K1s0Validators.phone(''), isNull);
      });

      test('returns error for invalid phone', () {
        expect(K1s0Validators.phone('123'), isNotNull);
        expect(K1s0Validators.phone('abc'), isNotNull);
      });

      test('returns null for valid Japanese phone', () {
        expect(K1s0Validators.phone('09012345678'), isNull);
        expect(K1s0Validators.phone('0312345678'), isNull);
        expect(K1s0Validators.phone('090-1234-5678'), isNull);
        expect(K1s0Validators.phone('+819012345678'), isNull);
      });
    });

    group('minLength', () {
      test('returns null for empty value', () {
        final validator = K1s0Validators.minLength(5);
        expect(validator(null), isNull);
        expect(validator(''), isNull);
      });

      test('returns error for short value', () {
        final validator = K1s0Validators.minLength(5);
        expect(validator('abc'), isNotNull);
        expect(validator('ab'), isNotNull);
      });

      test('returns null for valid length', () {
        final validator = K1s0Validators.minLength(5);
        expect(validator('abcde'), isNull);
        expect(validator('abcdef'), isNull);
      });
    });

    group('maxLength', () {
      test('returns null for empty value', () {
        final validator = K1s0Validators.maxLength(5);
        expect(validator(null), isNull);
        expect(validator(''), isNull);
      });

      test('returns error for long value', () {
        final validator = K1s0Validators.maxLength(5);
        expect(validator('abcdef'), isNotNull);
        expect(validator('abcdefghij'), isNotNull);
      });

      test('returns null for valid length', () {
        final validator = K1s0Validators.maxLength(5);
        expect(validator('abcde'), isNull);
        expect(validator('abc'), isNull);
      });
    });

    group('numeric', () {
      test('returns null for empty value', () {
        expect(K1s0Validators.numeric(null), isNull);
        expect(K1s0Validators.numeric(''), isNull);
      });

      test('returns error for non-numeric value', () {
        expect(K1s0Validators.numeric('abc'), isNotNull);
        expect(K1s0Validators.numeric('12.34.56'), isNotNull);
      });

      test('returns null for numeric value', () {
        expect(K1s0Validators.numeric('123'), isNull);
        expect(K1s0Validators.numeric('12.34'), isNull);
        expect(K1s0Validators.numeric('-123'), isNull);
        expect(K1s0Validators.numeric('0'), isNull);
      });
    });

    group('integer', () {
      test('returns null for empty value', () {
        expect(K1s0Validators.integer(null), isNull);
        expect(K1s0Validators.integer(''), isNull);
      });

      test('returns error for non-integer value', () {
        expect(K1s0Validators.integer('abc'), isNotNull);
        expect(K1s0Validators.integer('12.34'), isNotNull);
      });

      test('returns null for integer value', () {
        expect(K1s0Validators.integer('123'), isNull);
        expect(K1s0Validators.integer('-456'), isNull);
        expect(K1s0Validators.integer('0'), isNull);
      });
    });

    group('url', () {
      test('returns null for empty value', () {
        expect(K1s0Validators.url(null), isNull);
        expect(K1s0Validators.url(''), isNull);
      });

      test('returns error for invalid URL', () {
        expect(K1s0Validators.url('invalid'), isNotNull);
        expect(K1s0Validators.url('ftp://example.com'), isNotNull);
        expect(K1s0Validators.url('example.com'), isNotNull);
      });

      test('returns null for valid URL', () {
        expect(K1s0Validators.url('http://example.com'), isNull);
        expect(K1s0Validators.url('https://example.com'), isNull);
        expect(K1s0Validators.url('https://example.com/path?query=1'), isNull);
      });
    });

    group('pattern', () {
      test('returns null for empty value', () {
        final validator = K1s0Validators.pattern(RegExp(r'^\d+$'), 'Error');
        expect(validator(null), isNull);
        expect(validator(''), isNull);
      });

      test('returns error message for non-matching value', () {
        final validator =
            K1s0Validators.pattern(RegExp(r'^\d+$'), 'Numbers only');
        expect(validator('abc'), 'Numbers only');
      });

      test('returns null for matching value', () {
        final validator =
            K1s0Validators.pattern(RegExp(r'^\d+$'), 'Numbers only');
        expect(validator('123'), isNull);
      });
    });

    group('passwordStrength', () {
      test('returns null for empty value', () {
        expect(K1s0Validators.passwordStrength(null), isNull);
        expect(K1s0Validators.passwordStrength(''), isNull);
      });

      test('returns error for short password', () {
        expect(K1s0Validators.passwordStrength('Abc123'), isNotNull);
      });

      test('returns error for password without uppercase', () {
        expect(K1s0Validators.passwordStrength('abcdefgh1'), isNotNull);
      });

      test('returns error for password without lowercase', () {
        expect(K1s0Validators.passwordStrength('ABCDEFGH1'), isNotNull);
      });

      test('returns error for password without numbers', () {
        expect(K1s0Validators.passwordStrength('Abcdefghij'), isNotNull);
      });

      test('returns null for strong password', () {
        expect(K1s0Validators.passwordStrength('Abcdefg1'), isNull);
        expect(K1s0Validators.passwordStrength('Password123'), isNull);
      });
    });

    group('match', () {
      test('returns error when values do not match', () {
        final validator = K1s0Validators.match('password', 'Password');
        expect(validator('different'), isNotNull);
        expect(validator('different'), contains('Password'));
      });

      test('returns null when values match', () {
        final validator = K1s0Validators.match('password', 'Password');
        expect(validator('password'), isNull);
      });
    });

    group('combine', () {
      test('returns first error when multiple validators fail', () {
        final validator = K1s0Validators.combine([
          K1s0Validators.required,
          K1s0Validators.minLength(5),
        ]);

        expect(validator(''), isNotNull);
      });

      test('returns later error when first passes', () {
        final validator = K1s0Validators.combine([
          K1s0Validators.required,
          K1s0Validators.minLength(5),
        ]);

        final result = validator('abc');
        expect(result, isNotNull);
        expect(result, contains('5文字'));
      });

      test('returns null when all validators pass', () {
        final validator = K1s0Validators.combine([
          K1s0Validators.required,
          K1s0Validators.minLength(3),
          K1s0Validators.maxLength(10),
        ]);

        expect(validator('valid'), isNull);
      });
    });
  });
}
