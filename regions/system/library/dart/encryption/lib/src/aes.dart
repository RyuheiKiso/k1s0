import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:encrypt/encrypt.dart' as enc;

/// Generates a cryptographically secure random 256-bit (32-byte) AES key.
Uint8List generateKey() {
  final rng = Random.secure();
  return Uint8List.fromList(List.generate(32, (_) => rng.nextInt(256)));
}

/// Encrypts [plaintext] using AES-GCM with the given 256-bit [key].
///
/// A random 12-byte IV/nonce is generated for each call. The returned string
/// is a base64-encoded concatenation of:
///   IV (12 bytes) + ciphertext + GCM auth tag (16 bytes)
///
/// This format is self-contained: the [decrypt] function extracts the IV and
/// auth tag automatically.
String encrypt(Uint8List key, String plaintext) {
  final encrypter = enc.Encrypter(enc.AES(enc.Key(key), mode: enc.AESMode.gcm));
  final iv = enc.IV.fromSecureRandom(12);
  final encrypted = encrypter.encrypt(plaintext, iv: iv);

  // Concatenate: IV (12 bytes) + ciphertext + auth tag (16 bytes)
  final combined = Uint8List.fromList(iv.bytes + encrypted.bytes);
  return base64.encode(combined);
}

/// Decrypts [ciphertext] that was produced by [encrypt] using the same [key].
///
/// Extracts the 12-byte IV from the front and the 16-byte GCM auth tag from
/// the end, then decrypts and authenticates the middle portion.
///
/// Throws if the auth tag is invalid (i.e. the data was tampered with or the
/// wrong key was used).
String decrypt(Uint8List key, String ciphertext) {
  final combined = base64.decode(ciphertext);

  // Extract IV (first 12 bytes)
  final iv = enc.IV(Uint8List.fromList(combined.sublist(0, 12)));

  // The rest is ciphertext + GCM auth tag (handled by the encrypt package)
  final encryptedBytes = Uint8List.fromList(combined.sublist(12));
  final encrypted = enc.Encrypted(encryptedBytes);

  final encrypter = enc.Encrypter(enc.AES(enc.Key(key), mode: enc.AESMode.gcm));
  return encrypter.decrypt(encrypted, iv: iv);
}
