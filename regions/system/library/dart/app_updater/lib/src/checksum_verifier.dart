import 'dart:io';

import 'package:crypto/crypto.dart';

import 'error.dart';

class ChecksumVerifier {
  /// Verify SHA-256 checksum of a file.
  static Future<bool> verify(String filePath, String expectedChecksum) async {
    final actual = await calculate(filePath);
    return actual == expectedChecksum.toLowerCase();
  }

  static Future<void> verifyOrThrow(
    String filePath,
    String expectedChecksum,
  ) async {
    final verified = await verify(filePath, expectedChecksum);
    if (!verified) {
      throw const ChecksumError('Downloaded file checksum did not match.');
    }
  }

  /// Calculate SHA-256 checksum of a file.
  static Future<String> calculate(String filePath) async {
    final file = File(filePath);
    final digest = await sha256.bind(file.openRead()).last;
    return digest.toString();
  }
}
