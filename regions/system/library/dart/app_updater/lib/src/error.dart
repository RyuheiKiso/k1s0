class AppUpdaterError implements Exception {
  final String message;
  final String code;

  const AppUpdaterError(this.message, this.code);

  @override
  String toString() => 'AppUpdaterError($code): $message';
}

class ConnectionError extends AppUpdaterError {
  const ConnectionError(String message) : super(message, 'CONNECTION_ERROR');
}

class InvalidConfigError extends AppUpdaterError {
  const InvalidConfigError(String message) : super(message, 'INVALID_CONFIG');
}

class ParseError extends AppUpdaterError {
  const ParseError(String message) : super(message, 'PARSE_ERROR');
}

class UnauthorizedError extends AppUpdaterError {
  const UnauthorizedError(String message) : super(message, 'UNAUTHORIZED');
}

class AppNotFoundError extends AppUpdaterError {
  const AppNotFoundError(String message) : super(message, 'APP_NOT_FOUND');
}

class VersionNotFoundError extends AppUpdaterError {
  const VersionNotFoundError(String message)
      : super(message, 'VERSION_NOT_FOUND');
}

class StoreUrlUnavailableError extends AppUpdaterError {
  const StoreUrlUnavailableError(String message)
      : super(message, 'STORE_URL_UNAVAILABLE');
}

class ChecksumError extends AppUpdaterError {
  const ChecksumError(String message) : super(message, 'CHECKSUM_ERROR');
}

class SignatureVerificationError extends AppUpdaterError {
  const SignatureVerificationError(String message)
      : super(message, 'SIGNATURE_VERIFICATION_FAILED');
}

class DownloadError extends AppUpdaterError {
  const DownloadError(String message) : super(message, 'DOWNLOAD_ERROR');
}
