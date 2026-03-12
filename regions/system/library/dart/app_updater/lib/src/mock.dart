import 'app_updater.dart';
import 'model.dart';

class MockAppUpdater implements AppUpdater {
  final List<String> calls = <String>[];

  Future<AppVersionInfo> Function()? onFetchVersionInfo;
  Future<UpdateCheckResult> Function()? onCheckForUpdate;
  void Function(void Function(UpdateCheckResult result) onUpdateAvailable)?
      onStartPeriodicCheck;
  void Function()? onStopPeriodicCheck;
  String? Function()? onGetStoreUrl;
  Future<bool> Function()? onOpenStore;
  void Function()? onDispose;

  @override
  Future<AppVersionInfo> fetchVersionInfo() {
    calls.add('fetchVersionInfo');
    return onFetchVersionInfo?.call() ??
        Future<AppVersionInfo>.value(
          const AppVersionInfo(
            latestVersion: '0.0.0',
            minimumVersion: '0.0.0',
          ),
        );
  }

  @override
  Future<UpdateCheckResult> checkForUpdate() {
    calls.add('checkForUpdate');
    return onCheckForUpdate?.call() ??
        Future<UpdateCheckResult>.value(
          const UpdateCheckResult(
            type: UpdateType.none,
            currentVersion: '0.0.0',
            versionInfo: AppVersionInfo(
              latestVersion: '0.0.0',
              minimumVersion: '0.0.0',
            ),
          ),
        );
  }

  @override
  void startPeriodicCheck({
    required void Function(UpdateCheckResult result) onUpdateAvailable,
  }) {
    calls.add('startPeriodicCheck');
    onStartPeriodicCheck?.call(onUpdateAvailable);
  }

  @override
  void stopPeriodicCheck() {
    calls.add('stopPeriodicCheck');
    onStopPeriodicCheck?.call();
  }

  @override
  String? getStoreUrl() {
    calls.add('getStoreUrl');
    return onGetStoreUrl?.call();
  }

  @override
  Future<bool> openStore() {
    calls.add('openStore');
    return onOpenStore?.call() ?? Future<bool>.value(false);
  }

  @override
  void dispose() {
    calls.add('dispose');
    onDispose?.call();
  }
}
