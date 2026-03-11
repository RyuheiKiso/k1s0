import 'package:meta/meta.dart';

enum UpdateType {
  none,
  optional,
  mandatory,
}

@immutable
class AppVersionInfo {
  final String latestVersion;
  final String minimumVersion;
  final String? releaseNotes;
  final bool mandatory;
  final String? storeUrl;
  final DateTime? publishedAt;
  final String? platform;
  final String? arch;
  final int? sizeBytes;
  final String? checksumSha256;
  final String? downloadUrl;
  final int? expiresIn;

  const AppVersionInfo({
    required this.latestVersion,
    required this.minimumVersion,
    this.releaseNotes,
    this.mandatory = false,
    this.storeUrl,
    this.publishedAt,
    this.platform,
    this.arch,
    this.sizeBytes,
    this.checksumSha256,
    this.downloadUrl,
    this.expiresIn,
  });

  AppVersionInfo copyWith({
    String? latestVersion,
    String? minimumVersion,
    String? releaseNotes,
    bool? mandatory,
    String? storeUrl,
    DateTime? publishedAt,
    String? platform,
    String? arch,
    int? sizeBytes,
    String? checksumSha256,
    String? downloadUrl,
    int? expiresIn,
  }) {
    return AppVersionInfo(
      latestVersion: latestVersion ?? this.latestVersion,
      minimumVersion: minimumVersion ?? this.minimumVersion,
      releaseNotes: releaseNotes ?? this.releaseNotes,
      mandatory: mandatory ?? this.mandatory,
      storeUrl: storeUrl ?? this.storeUrl,
      publishedAt: publishedAt ?? this.publishedAt,
      platform: platform ?? this.platform,
      arch: arch ?? this.arch,
      sizeBytes: sizeBytes ?? this.sizeBytes,
      checksumSha256: checksumSha256 ?? this.checksumSha256,
      downloadUrl: downloadUrl ?? this.downloadUrl,
      expiresIn: expiresIn ?? this.expiresIn,
    );
  }
}

@immutable
class DownloadArtifactInfo {
  final String downloadUrl;
  final int expiresIn;
  final String checksumSha256;
  final int? sizeBytes;

  const DownloadArtifactInfo({
    required this.downloadUrl,
    required this.expiresIn,
    required this.checksumSha256,
    this.sizeBytes,
  });
}

@immutable
class UpdateCheckResult {
  final UpdateType type;
  final String currentVersion;
  final AppVersionInfo versionInfo;

  const UpdateCheckResult({
    required this.type,
    required this.currentVersion,
    required this.versionInfo,
  });

  bool get needsUpdate => type != UpdateType.none;

  bool get isMandatory => type == UpdateType.mandatory;
}
