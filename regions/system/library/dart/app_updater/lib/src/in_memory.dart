import 'dart:async';

import 'app_updater.dart';
import 'model.dart';

class InMemoryAppUpdater implements AppUpdater {
  AppVersionInfo _versionInfo;
  String _currentVersion;
  final Future<bool> Function(Uri uri)? _storeOpener;

  Timer? _periodicTimer;

  InMemoryAppUpdater({
    required AppVersionInfo versionInfo,
    required String currentVersion,
    Future<bool> Function(Uri uri)? storeOpener,
  })  : _versionInfo = versionInfo,
        _currentVersion = currentVersion,
        _storeOpener = storeOpener;

  @override
  Future<AppVersionInfo> fetchVersionInfo() async => _versionInfo;

  @override
  Future<UpdateCheckResult> checkForUpdate() async {
    return UpdateCheckResult(
      type: determineUpdateType(
        currentVersion: _currentVersion,
        versionInfo: _versionInfo,
      ),
      currentVersion: _currentVersion,
      versionInfo: _versionInfo,
    );
  }

  @override
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  }) {
    _periodicTimer?.cancel();
    _periodicTimer =
        Timer.periodic(const Duration(milliseconds: 100), (_) async {
      final result = await checkForUpdate();
      if (result.needsUpdate) {
        onUpdateAvailable(result);
      }
    });
  }

  @override
  void stopPeriodicCheck() {
    _periodicTimer?.cancel();
    _periodicTimer = null;
  }

  @override
  String? getStoreUrl() => _versionInfo.storeUrl;

  @override
  Future<bool> openStore() async {
    final storeUrl = getStoreUrl();
    final storeOpener = _storeOpener;
    if (storeUrl == null || storeOpener == null) {
      return false;
    }
    return storeOpener(Uri.parse(storeUrl));
  }

  void setVersionInfo(AppVersionInfo info) {
    _versionInfo = info;
  }

  void setCurrentVersion(String version) {
    _currentVersion = version;
  }

  @override
  void dispose() {
    stopPeriodicCheck();
  }
}
