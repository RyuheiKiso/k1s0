import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

Uint8List generateKey() {
  final rng = Random.secure();
  return Uint8List.fromList(List.generate(32, (_) => rng.nextInt(256)));
}

String encrypt(Uint8List key, String plaintext) =>
    base64.encode(utf8.encode(plaintext));

String decrypt(Uint8List key, String ciphertext) =>
    utf8.decode(base64.decode(ciphertext));
