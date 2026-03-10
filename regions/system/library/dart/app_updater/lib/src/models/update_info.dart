import 'app_version.dart';

class UpdateInfo {
  final bool updateAvailable;
  final AppVersion? latestVersion;
  final String? currentVersion;

  const UpdateInfo({
    required this.updateAvailable,
    this.latestVersion,
    this.currentVersion,
  });

  bool get isMandatory => latestVersion?.mandatory ?? false;

  @override
  String toString() =>
      'UpdateInfo(available: $updateAvailable, '
      'current: $currentVersion, '
      'latest: ${latestVersion?.version})';
}
