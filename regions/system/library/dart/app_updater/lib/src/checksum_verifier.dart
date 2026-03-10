import 'dart:io';

import 'package:crypto/crypto.dart';

class ChecksumVerifier {
  /// Verify SHA-256 checksum of a file.
  static Future<bool> verify(String filePath, String expectedChecksum) async {
    final actual = await calculate(filePath);
    return actual == expectedChecksum.toLowerCase();
  }

  /// Calculate SHA-256 checksum of a file.
  static Future<String> calculate(String filePath) async {
    final file = File(filePath);
    final digest = await sha256.bind(file.openRead()).last;
    return digest.toString();
  }
}
