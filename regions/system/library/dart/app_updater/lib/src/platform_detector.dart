import 'dart:io';

/// 実行環境のプラットフォームとアーキテクチャを検出するユーティリティ。
/// dart:io の Platform クラスをラップし、文字列形式で情報を返す。
class PlatformDetector {
  /// 現在のプラットフォームを返す。
  /// 'windows', 'linux', 'macos', 'android', 'ios' のいずれかを返す。
  /// 非対応プラットフォームの場合は [UnsupportedError] をスローする。
  static String get currentPlatform {
    if (Platform.isWindows) return 'windows';
    if (Platform.isLinux) return 'linux';
    if (Platform.isMacOS) return 'macos';
    if (Platform.isAndroid) return 'android';
    if (Platform.isIOS) return 'ios';
    throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
  }

  /// 現在のアーキテクチャを返す。
  /// 'amd64' または 'arm64' を返す。
  /// 判定できないアーキテクチャの場合は [UnsupportedError] をスローする。
  static String get currentArch {
    final arch = _resolveArch();
    if (arch == 'amd64' || arch == 'arm64') return arch;
    throw UnsupportedError('Unsupported architecture: $arch');
  }

  /// 実行可能ファイルのパスからアーキテクチャを推定する。
  /// 'arm64' または 'aarch64' が含まれる場合は 'arm64'、それ以外は 'amd64' を返す。
  static String _resolveArch() {
    final executable = Platform.resolvedExecutable.toLowerCase();
    if (executable.contains('arm64') || executable.contains('aarch64')) {
      return 'arm64';
    }
    // arm64/aarch64 が含まれない場合は x86_64 (amd64) と判断する。
    return 'amd64';
  }
}
