import 'dart:io';

import 'package:test/test.dart';

import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  group('currentPlatform', () {
    test('returns a valid platform string', () {
      final platform = PlatformDetector.currentPlatform;
      expect(
        platform,
        anyOf(equals('windows'), equals('linux'), equals('macos')),
      );
    });

    test('matches dart Platform', () {
      final platform = PlatformDetector.currentPlatform;
      if (Platform.isWindows) {
        expect(platform, equals('windows'));
      } else if (Platform.isLinux) {
        expect(platform, equals('linux'));
      } else if (Platform.isMacOS) {
        expect(platform, equals('macos'));
      }
    });
  });

  group('currentArch', () {
    test('returns a valid architecture string', () {
      final arch = PlatformDetector.currentArch;
      expect(arch, anyOf(equals('x64'), equals('arm64')));
    });
  });
}
