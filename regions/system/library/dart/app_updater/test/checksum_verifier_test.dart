import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:test/test.dart';

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

  group('calculate', () {
    test('returns correct SHA-256 for file', () async {
      final checksum = await ChecksumVerifier.calculate(testFilePath);

      // SHA-256 of 'hello world'
      final expected = sha256.convert('hello world'.codeUnits).toString();
      expect(checksum, equals(expected));
    });
  });

  group('verify', () {
    test('returns true for correct checksum', () async {
      final expected = sha256.convert('hello world'.codeUnits).toString();
      final result = await ChecksumVerifier.verify(testFilePath, expected);
      expect(result, isTrue);
    });

    test('returns true for uppercase checksum', () async {
      final expected =
          sha256.convert('hello world'.codeUnits).toString().toUpperCase();
      final result = await ChecksumVerifier.verify(testFilePath, expected);
      expect(result, isTrue);
    });

    test('returns false for incorrect checksum', () async {
      final result = await ChecksumVerifier.verify(
        testFilePath,
        'incorrect_checksum_value',
      );
      expect(result, isFalse);
    });
  });
}
