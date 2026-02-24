import 'dart:math';
import 'dart:typed_data';

import 'package:hashlib/hashlib.dart';

/// Argon2id parameters matching other k1s0 libraries.
const int _memoryCostKB = 19456;
const int _iterations = 2;
const int _parallelism = 1;
const int _hashLength = 32;
const int _saltLength = 16;

/// Hash a password with Argon2id.
///
/// Returns a PHC-format string:
/// `$argon2id$v=19$m=19456,t=2,p=1$<salt_base64>$<hash_base64>`
String hashPassword(String password) {
  final salt = _generateRandomBytes(_saltLength);

  final argon2 = Argon2(
    type: Argon2Type.argon2id,
    version: Argon2Version.v13,
    memorySizeKB: _memoryCostKB,
    iterations: _iterations,
    parallelism: _parallelism,
    hashLength: _hashLength,
    salt: salt,
  );

  return argon2.encode(password.codeUnits);
}

/// Verify a password against an Argon2id hash string.
bool verifyPassword(String password, String hash) {
  try {
    return argon2Verify(hash, password.codeUnits);
  } catch (_) {
    return false;
  }
}

Uint8List _generateRandomBytes(int length) {
  final random = Random.secure();
  return Uint8List.fromList(
      List<int>.generate(length, (_) => random.nextInt(256)));
}
