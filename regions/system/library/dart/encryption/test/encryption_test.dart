import 'dart:convert';
import 'dart:typed_data';

import 'package:test/test.dart';

import 'package:k1s0_encryption/encryption.dart';

void main() {
  group('hashPassword', () {
    test('produces argon2id format hash', () {
      final hash = hashPassword('secret');
      expect(hash, startsWith(r'$argon2id$'));
      expect(hash, contains('m=19456,t=2,p=1'));
    });

    test('different calls produce different hashes (random salt)', () {
      final h1 = hashPassword('secret');
      final h2 = hashPassword('secret');
      expect(h1, isNot(equals(h2)));
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

    test('returns false for invalid hash format', () {
      expect(verifyPassword('test', 'not-a-valid-hash'), isFalse);
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

    test('ciphertext is not the plaintext in base64', () {
      final key = generateKey();
      final message = 'hello world';
      final ciphertext = encrypt(key, message);
      // The old fake implementation would produce the same base64
      final naiveBase64 = base64.encode(utf8.encode(message));
      expect(ciphertext, isNot(equals(naiveBase64)));
    });

    test('encrypting the same plaintext twice produces different ciphertext', () {
      final key = generateKey();
      final c1 = encrypt(key, 'same message');
      final c2 = encrypt(key, 'same message');
      // Different random IVs should produce different ciphertext
      expect(c1, isNot(equals(c2)));
    });

    test('decryption with wrong key throws', () {
      final key1 = generateKey();
      final key2 = generateKey();
      final ciphertext = encrypt(key1, 'secret data');
      expect(() => decrypt(key2, ciphertext), throwsA(anything));
    });

    test('handles empty string', () {
      final key = generateKey();
      final ciphertext = encrypt(key, '');
      final plaintext = decrypt(key, ciphertext);
      expect(plaintext, equals(''));
    });

    test('handles unicode content', () {
      final key = generateKey();
      final message = 'こんにちは世界 🌍';
      final ciphertext = encrypt(key, message);
      final plaintext = decrypt(key, ciphertext);
      expect(plaintext, equals(message));
    });

    test('tampered ciphertext fails authentication', () {
      final key = generateKey();
      final ciphertext = encrypt(key, 'important data');
      final bytes = base64.decode(ciphertext);
      // Flip a byte in the ciphertext portion (after the 12-byte IV)
      bytes[15] ^= 0xFF;
      final tampered = base64.encode(bytes);
      expect(() => decrypt(key, tampered), throwsA(anything));
    });
  });
}
