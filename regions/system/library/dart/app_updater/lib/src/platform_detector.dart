import 'dart:io';

class PlatformDetector {
  /// Detect current platform (windows/linux/macos).
  static String get currentPlatform {
    if (Platform.isWindows) return 'windows';
    if (Platform.isLinux) return 'linux';
    if (Platform.isMacOS) return 'macos';
    throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
  }

  /// Detect current architecture (x64/arm64).
  static String get currentArch {
    final arch = _resolveArch();
    if (arch == 'x64' || arch == 'arm64') return arch;
    throw UnsupportedError('Unsupported architecture: $arch');
  }

  static String _resolveArch() {
    // Dart's Platform doesn't expose architecture directly.
    // We use the Dart executable path heuristic and process info.
    final executable = Platform.resolvedExecutable.toLowerCase();
    if (executable.contains('arm64') || executable.contains('aarch64')) {
      return 'arm64';
    }
    // Default to x64 for desktop platforms.
    return 'x64';
  }
}
