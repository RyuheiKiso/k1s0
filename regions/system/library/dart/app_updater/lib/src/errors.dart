class AppUpdaterError implements Exception {
  final String message;
  final String code;

  const AppUpdaterError(this.message, this.code);

  @override
  String toString() => 'AppUpdaterError($code): $message';
}

class NetworkError extends AppUpdaterError {
  const NetworkError(String message) : super(message, 'NETWORK_ERROR');
}

class ChecksumError extends AppUpdaterError {
  const ChecksumError(String message) : super(message, 'CHECKSUM_ERROR');
}

class VersionNotFoundError extends AppUpdaterError {
  const VersionNotFoundError(String message)
      : super(message, 'VERSION_NOT_FOUND');
}

class DownloadError extends AppUpdaterError {
  const DownloadError(String message) : super(message, 'DOWNLOAD_ERROR');
}

class ApplyError extends AppUpdaterError {
  const ApplyError(String message) : super(message, 'APPLY_ERROR');
}
