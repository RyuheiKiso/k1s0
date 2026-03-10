class AppVersion {
  final String appId;
  final String version;
  final String platform;
  final String arch;
  final int? sizeBytes;
  final String checksumSha256;
  final String? releaseNotes;
  final bool mandatory;
  final DateTime publishedAt;
  final String? downloadUrl;

  const AppVersion({
    required this.appId,
    required this.version,
    required this.platform,
    required this.arch,
    this.sizeBytes,
    required this.checksumSha256,
    this.releaseNotes,
    required this.mandatory,
    required this.publishedAt,
    this.downloadUrl,
  });

  factory AppVersion.fromJson(Map<String, dynamic> json) {
    return AppVersion(
      appId: json['app_id'] as String,
      version: json['version'] as String,
      platform: json['platform'] as String,
      arch: json['arch'] as String,
      sizeBytes: json['size_bytes'] as int?,
      checksumSha256: json['checksum_sha256'] as String,
      releaseNotes: json['release_notes'] as String?,
      mandatory: json['mandatory'] as bool,
      publishedAt: DateTime.parse(json['published_at'] as String),
      downloadUrl: json['download_url'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'app_id': appId,
      'version': version,
      'platform': platform,
      'arch': arch,
      if (sizeBytes != null) 'size_bytes': sizeBytes,
      'checksum_sha256': checksumSha256,
      if (releaseNotes != null) 'release_notes': releaseNotes,
      'mandatory': mandatory,
      'published_at': publishedAt.toIso8601String(),
      if (downloadUrl != null) 'download_url': downloadUrl,
    };
  }

  @override
  String toString() => 'AppVersion($appId@$version, $platform/$arch)';
}
