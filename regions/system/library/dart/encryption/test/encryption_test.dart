import 'package:test/test.dart';

import 'package:k1s0_encryption/encryption.dart';

void main() {
  group('hashPassword', () {
    test('produces consistent hash', () {
      final h1 = hashPassword('secret');
      final h2 = hashPassword('secret');
      expect(h1, equals(h2));
    });

    test('different passwords produce different hashes', () {
      final h1 = hashPassword('password1');
      final h2 = hashPassword('password2');
      expect(h1, isNot(equals(h2)));
    });
  });

  group('verifyPassword', () {
    test('returns true for matching password', () {
      final hash = hashPassword('mypassword');
      expect(verifyPassword('mypassword', hash), isTrue);
    });

    test('returns false for wrong password', () {
      final hash = hashPassword('correct');
      expect(verifyPassword('wrong', hash), isFalse);
    });
  });

  group('generateKey', () {
    test('returns 32 bytes', () {
      final key = generateKey();
      expect(key.length, equals(32));
    });

    test('generates different keys', () {
      final k1 = generateKey();
      final k2 = generateKey();
      expect(k1, isNot(equals(k2)));
    });
  });

  group('encrypt/decrypt', () {
    test('round-trip preserves plaintext', () {
      final key = generateKey();
      final ciphertext = encrypt(key, 'hello world');
      final plaintext = decrypt(key, ciphertext);
      expect(plaintext, equals('hello world'));
    });
  });
}
