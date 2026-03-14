import 'dart:convert';

import 'package:test/test.dart';

import 'package:k1s0_encryption/encryption.dart';

void main() {
  group('hashPassword', () {
    test('argon2id形式のハッシュを生成すること', () {
      final hash = hashPassword('secret');
      expect(hash, startsWith(r'$argon2id$'));
      expect(hash, contains('m=19456,t=2,p=1'));
    });

    test('呼び出しごとに異なるハッシュを生成すること（ランダムソルト）', () {
      final h1 = hashPassword('secret');
      final h2 = hashPassword('secret');
      expect(h1, isNot(equals(h2)));
    });

    test('異なるパスワードは異なるハッシュを生成すること', () {
      final h1 = hashPassword('password1');
      final h2 = hashPassword('password2');
      expect(h1, isNot(equals(h2)));
    });
  });

  group('verifyPassword', () {
    test('一致するパスワードでtrueを返すこと', () {
      final hash = hashPassword('mypassword');
      expect(verifyPassword('mypassword', hash), isTrue);
    });

    test('誤ったパスワードでfalseを返すこと', () {
      final hash = hashPassword('correct');
      expect(verifyPassword('wrong', hash), isFalse);
    });

    test('不正なハッシュ形式でfalseを返すこと', () {
      expect(verifyPassword('test', 'not-a-valid-hash'), isFalse);
    });
  });

  group('generateKey', () {
    test('32バイトのキーを返すこと', () {
      final key = generateKey();
      expect(key.length, equals(32));
    });

    test('呼び出しごとに異なるキーを生成すること', () {
      final k1 = generateKey();
      final k2 = generateKey();
      expect(k1, isNot(equals(k2)));
    });
  });

  group('encrypt/decrypt', () {
    test('暗号化・復号のラウンドトリップで平文が保持されること', () {
      final key = generateKey();
      final ciphertext = encrypt(key, 'hello world');
      final plaintext = decrypt(key, ciphertext);
      expect(plaintext, equals('hello world'));
    });

    test('暗号文が平文のbase64エンコードと異なること', () {
      final key = generateKey();
      final message = 'hello world';
      final ciphertext = encrypt(key, message);
      // The old fake implementation would produce the same base64
      final naiveBase64 = base64.encode(utf8.encode(message));
      expect(ciphertext, isNot(equals(naiveBase64)));
    });

    test('同じ平文を2回暗号化すると異なる暗号文が生成されること', () {
      final key = generateKey();
      final c1 = encrypt(key, 'same message');
      final c2 = encrypt(key, 'same message');
      // Different random IVs should produce different ciphertext
      expect(c1, isNot(equals(c2)));
    });

    test('誤ったキーで復号するとエラーがスローされること', () {
      final key1 = generateKey();
      final key2 = generateKey();
      final ciphertext = encrypt(key1, 'secret data');
      expect(() => decrypt(key2, ciphertext), throwsA(anything));
    });

    test('空文字列を正常に処理できること', () {
      final key = generateKey();
      final ciphertext = encrypt(key, '');
      final plaintext = decrypt(key, ciphertext);
      expect(plaintext, equals(''));
    });

    test('ユニコード文字列を正常に処理できること', () {
      final key = generateKey();
      final message = 'こんにちは世界 🌍';
      final ciphertext = encrypt(key, message);
      final plaintext = decrypt(key, ciphertext);
      expect(plaintext, equals(message));
    });

    test('改ざんされた暗号文は認証に失敗すること', () {
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
