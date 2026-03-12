import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  late Directory tempDir;
  late String testFilePath;

  setUp(() async {
    tempDir = await Directory.systemTemp.createTemp('checksum_test_');
    testFilePath = '${tempDir.path}/test_file.bin';
    await File(testFilePath).writeAsString('hello world');
  });

  tearDown(() async {
    await tempDir.delete(recursive: true);
  });

  test('calculates the SHA-256 checksum for a file', () async {
    final checksum = await ChecksumVerifier.calculate(testFilePath);
    final expected = sha256.convert('hello world'.codeUnits).toString();

    expect(checksum, expected);
  });

  test('verifyOrThrow throws when the checksum is invalid', () async {
    expect(
      () => ChecksumVerifier.verifyOrThrow(testFilePath, 'invalid'),
      throwsA(isA<ChecksumError>()),
    );
  });
}
