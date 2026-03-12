import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_app_updater/app_updater.dart';

void main() {
  test('currentPlatform matches dart:io platform', () {
    final platform = PlatformDetector.currentPlatform;

    if (Platform.isWindows) {
      expect(platform, 'windows');
    } else if (Platform.isLinux) {
      expect(platform, 'linux');
    } else if (Platform.isMacOS) {
      expect(platform, 'macos');
    }
  });

  test('currentArch is normalized', () {
    expect(PlatformDetector.currentArch, anyOf('amd64', 'arm64'));
  });
}
