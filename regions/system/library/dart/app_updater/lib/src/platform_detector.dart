import 'dart:io';

class PlatformDetector {
  /// Detect current platform.
  static String get currentPlatform {
    if (Platform.isWindows) return 'windows';
    if (Platform.isLinux) return 'linux';
    if (Platform.isMacOS) return 'macos';
    if (Platform.isAndroid) return 'android';
    if (Platform.isIOS) return 'ios';
    throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
  }

  /// Detect current architecture.
  static String get currentArch {
    final arch = _resolveArch();
    if (arch == 'amd64' || arch == 'arm64') return arch;
    throw UnsupportedError('Unsupported architecture: $arch');
  }

  static String _resolveArch() {
    final executable = Platform.resolvedExecutable.toLowerCase();
    if (executable.contains('arm64') || executable.contains('aarch64')) {
      return 'arm64';
    }
    return 'amd64';
  }
}
