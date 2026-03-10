import 'dart:io';

import 'package:path/path.dart' as p;

import 'checksum_verifier.dart';
import 'errors.dart';
import 'models/app_version.dart';
import 'models/update_info.dart';
import 'platform_detector.dart';
import 'registry_api_client.dart';

/// Main auto-update class for desktop applications.
class AppUpdater {
  final String appId;
  final String currentVersion;
  final RegistryApiClient _client;
  final String _downloadDir;

  AppUpdater({
    required this.appId,
    required this.currentVersion,
    required String registryUrl,
    String? authToken,
    String? downloadDir,
    RegistryApiClient? client,
  })  : _client = client ??
            RegistryApiClient(
              baseUrl: registryUrl,
              authToken: authToken,
            ),
        _downloadDir = downloadDir ?? Directory.systemTemp.path;

  /// Check if an update is available.
  Future<UpdateInfo> checkForUpdate() async {
    final platform = PlatformDetector.currentPlatform;
    final arch = PlatformDetector.currentArch;

    final latest = await _client.getLatestVersion(appId, platform, arch);
    if (latest == null) {
      return UpdateInfo(
        updateAvailable: false,
        currentVersion: currentVersion,
      );
    }

    final isNewer = _isNewerVersion(latest.version, currentVersion);
    return UpdateInfo(
      updateAvailable: isNewer,
      latestVersion: latest,
      currentVersion: currentVersion,
    );
  }

  /// Download the latest version.
  Future<String> download(
    AppVersion version, {
    void Function(int received, int total)? onProgress,
  }) async {
    final url = version.downloadUrl ??
        await _client.getDownloadUrl(
          appId,
          version.version,
          version.platform,
          version.arch,
        );

    final uri = Uri.parse(url);
    final request = await HttpClient().getUrl(uri);
    final response = await request.close();

    if (response.statusCode != 200) {
      throw DownloadError('HTTP ${response.statusCode}');
    }

    final fileName = uri.pathSegments.isNotEmpty
        ? uri.pathSegments.last
        : '${version.appId}-${version.version}';
    final filePath = p.join(_downloadDir, fileName);
    final file = File(filePath);
    final sink = file.openWrite();

    final total = response.contentLength;
    var received = 0;

    await for (final chunk in response) {
      sink.add(chunk);
      received += chunk.length;
      onProgress?.call(received, total);
    }

    await sink.close();
    return filePath;
  }

  /// Verify downloaded file checksum.
  Future<bool> verify(String filePath, String expectedChecksum) async {
    return ChecksumVerifier.verify(filePath, expectedChecksum);
  }

  /// Apply update and restart (platform-specific).
  Future<void> applyAndRestart(String installerPath) async {
    final file = File(installerPath);
    if (!await file.exists()) {
      throw ApplyError('Installer not found: $installerPath');
    }

    final platform = PlatformDetector.currentPlatform;

    switch (platform) {
      case 'windows':
        await Process.start(
          installerPath,
          ['/SILENT', '/RESTARTAPPLICATIONS'],
          mode: ProcessStartMode.detached,
        );
      case 'macos':
        await Process.start(
          'open',
          [installerPath],
          mode: ProcessStartMode.detached,
        );
      case 'linux':
        await Process.start(
          'chmod',
          ['+x', installerPath],
        );
        await Process.start(
          installerPath,
          [],
          mode: ProcessStartMode.detached,
        );
      default:
        throw ApplyError('Unsupported platform: $platform');
    }

    exit(0);
  }

  /// Compare semantic versions. Returns true if [newer] > [current].
  bool _isNewerVersion(String newer, String current) {
    final newerParts = newer.split('.').map(int.tryParse).toList();
    final currentParts = current.split('.').map(int.tryParse).toList();

    for (var i = 0; i < newerParts.length && i < currentParts.length; i++) {
      final n = newerParts[i] ?? 0;
      final c = currentParts[i] ?? 0;
      if (n > c) return true;
      if (n < c) return false;
    }
    return newerParts.length > currentParts.length;
  }
}
