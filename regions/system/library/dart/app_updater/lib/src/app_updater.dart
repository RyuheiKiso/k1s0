import 'dart:async';

import 'package:package_info_plus/package_info_plus.dart';
import 'package:url_launcher/url_launcher.dart';

import 'config.dart';
import 'error.dart';
import 'model.dart';
import 'platform_detector.dart';
import 'registry_api_client.dart';

abstract class AppUpdater {
  Future<AppVersionInfo> fetchVersionInfo();

  Future<UpdateCheckResult> checkForUpdate();

  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  });

  void stopPeriodicCheck();

  String? getStoreUrl();

  Future<bool> openStore();

  void dispose();
}

class AppRegistryAppUpdater implements AppUpdater {
  final AppUpdaterConfig _config;
  final RegistryApiClient _client;
  final Future<String> Function() _currentVersionProvider;
  final Future<bool> Function(Uri uri) _launchUrl;

  Timer? _periodicTimer;

  AppRegistryAppUpdater(
    this._config, {
    RegistryApiClient? client,
  })  : _client = client ??
            RegistryApiClient(
              baseUrl: _config.serverUrl,
              client: _config.httpClient,
              timeout: _config.timeout,
              tokenProvider: _config.tokenProvider,
            ),
        _currentVersionProvider =
            _config.currentVersionProvider ?? _defaultCurrentVersionProvider,
        _launchUrl = _config.urlLauncher ?? launchUrl {
    _validateConfig(_config);
  }

  @override
  Future<AppVersionInfo> fetchVersionInfo() async {
    final platform = _resolvePlatform();
    final arch = _resolveArch();

    final latest = await _client.getLatestVersion(
      _config.appId,
      platform: platform,
      arch: arch,
    );
    final versions = await _client.listVersions(_config.appId);

    final mandatoryVersions =
        versions.where((version) => version.mandatory).toList()
          ..sort((left, right) {
            final leftPublishedAt =
                left.publishedAt ?? DateTime.fromMillisecondsSinceEpoch(0);
            final rightPublishedAt =
                right.publishedAt ?? DateTime.fromMillisecondsSinceEpoch(0);
            return rightPublishedAt.compareTo(leftPublishedAt);
          });

    final minimumVersion = mandatoryVersions.isNotEmpty
        ? mandatoryVersions.first.version
        : latest.mandatory
            ? latest.version
            : '0.0.0';

    return AppVersionInfo(
      latestVersion: latest.version,
      minimumVersion: minimumVersion,
      releaseNotes: latest.releaseNotes,
      mandatory: latest.mandatory,
      storeUrl: getStoreUrl(),
      publishedAt: latest.publishedAt,
      platform: latest.platform,
      arch: latest.arch,
      sizeBytes: latest.sizeBytes,
      checksumSha256: latest.checksumSha256,
      downloadUrl: latest.downloadUrl,
    );
  }

  Future<DownloadArtifactInfo> fetchDownloadInfo({
    String? version,
    String? platform,
    String? arch,
  }) async {
    final resolvedInfo = version == null ? await fetchVersionInfo() : null;
    return _client.getDownloadInfo(
      _config.appId,
      version ?? resolvedInfo!.latestVersion,
      platform: platform ?? _resolvePlatform(),
      arch: arch ?? _resolveArch(),
    );
  }

  @override
  Future<UpdateCheckResult> checkForUpdate() async {
    final currentVersion = await _currentVersionProvider();
    final versionInfo = await fetchVersionInfo();
    final updateType = determineUpdateType(
      currentVersion: currentVersion,
      versionInfo: versionInfo,
    );

    return UpdateCheckResult(
      type: updateType,
      currentVersion: currentVersion,
      versionInfo: versionInfo,
    );
  }

  @override
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  }) {
    final interval = _config.checkInterval;
    if (interval == null) {
      return;
    }

    _periodicTimer?.cancel();
    _periodicTimer = Timer.periodic(interval, (_) async {
      try {
        final result = await checkForUpdate();
        if (result.needsUpdate) {
          onUpdateAvailable(result);
        }
      } on AppUpdaterError {
        // Ignore background polling failures.
      }
    });
  }

  @override
  void stopPeriodicCheck() {
    _periodicTimer?.cancel();
    _periodicTimer = null;
  }

  @override
  String? getStoreUrl() {
    final platform = (_config.platform ?? _safePlatform())?.toLowerCase();
    if (platform == null) {
      return null;
    }
    switch (platform) {
      case 'ios':
        return _config.iosStoreUrl;
      case 'android':
        return _config.androidStoreUrl;
      default:
        return null;
    }
  }

  @override
  Future<bool> openStore() async {
    final storeUrl = getStoreUrl();
    if (storeUrl == null) {
      return false;
    }
    return _launchUrl(Uri.parse(storeUrl));
  }

  @override
  void dispose() {
    stopPeriodicCheck();
  }

  String? _resolvePlatform() {
    return (_config.platform ?? _safePlatform())?.toLowerCase();
  }

  String? _safePlatform() {
    try {
      return PlatformDetector.currentPlatform;
    } on UnsupportedError {
      return null;
    }
  }

  String? _resolveArch() {
    if (_config.arch != null && _config.arch!.isNotEmpty) {
      return _config.arch;
    }

    try {
      return PlatformDetector.currentArch;
    } on UnsupportedError {
      return null;
    }
  }
}

Future<String> _defaultCurrentVersionProvider() async {
  final packageInfo = await PackageInfo.fromPlatform();
  return packageInfo.version;
}

void _validateConfig(AppUpdaterConfig config) {
  if (config.serverUrl.trim().isEmpty) {
    throw const InvalidConfigError('serverUrl must not be empty.');
  }
  if (config.appId.trim().isEmpty) {
    throw const InvalidConfigError('appId must not be empty.');
  }
}

UpdateType determineUpdateType({
  required String currentVersion,
  required AppVersionInfo versionInfo,
}) {
  if (_compareVersions(currentVersion, versionInfo.minimumVersion) < 0 ||
      versionInfo.mandatory) {
    return UpdateType.mandatory;
  }

  if (_compareVersions(currentVersion, versionInfo.latestVersion) < 0) {
    return UpdateType.optional;
  }

  return UpdateType.none;
}

int _compareVersions(String left, String right) {
  final leftParts = _normalizeVersion(left);
  final rightParts = _normalizeVersion(right);
  final length = leftParts.length > rightParts.length
      ? leftParts.length
      : rightParts.length;

  for (var index = 0; index < length; index += 1) {
    final leftValue = index < leftParts.length ? leftParts[index] : 0;
    final rightValue = index < rightParts.length ? rightParts[index] : 0;
    if (leftValue != rightValue) {
      return leftValue.compareTo(rightValue);
    }
  }

  return 0;
}

List<int> _normalizeVersion(String version) {
  return version
      .split('.')
      .map((segment) =>
          int.tryParse(segment.replaceAll(RegExp(r'[^0-9]'), '')) ?? 0)
      .toList();
}
