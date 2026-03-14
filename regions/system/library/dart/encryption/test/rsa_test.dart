import 'dart:typed_data';
import 'package:test/test.dart';
import 'package:k1s0_encryption/encryption.dart';

void main() {
  group('RSA', () {
    test('暗号化・復号のラウンドトリップが正常に動作すること', () {
      final keyPair = generateRsaKeyPair();
      final plaintext = Uint8List.fromList('hello RSA-OAEP'.codeUnits);
      final ciphertext = rsaEncrypt(keyPair['publicKey']!, plaintext);
      final decrypted = rsaDecrypt(keyPair['privateKey']!, ciphertext);
      expect(String.fromCharCodes(decrypted), equals('hello RSA-OAEP'));
    });

    test('誤ったキーで復号するとエラーがスローされること', () {
      final keyPair1 = generateRsaKeyPair();
      final keyPair2 = generateRsaKeyPair();
      final plaintext = Uint8List.fromList('secret'.codeUnits);
      final ciphertext = rsaEncrypt(keyPair1['publicKey']!, plaintext);
      expect(
        () => rsaDecrypt(keyPair2['privateKey']!, ciphertext),
        throwsA(anything),
      );
    });
  });
}
