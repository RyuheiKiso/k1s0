class MigrationError implements Exception {
  final String message;
  final String? code;
  final Object? cause;

  const MigrationError(this.message, {this.code, this.cause});

  factory MigrationError.connectionFailed(String message, {Object? cause}) =>
      MigrationError(message, code: 'CONNECTION_FAILED', cause: cause);

  factory MigrationError.migrationFailed(String version, String message,
          {Object? cause}) =>
      MigrationError('Migration $version failed: $message',
          code: 'MIGRATION_FAILED', cause: cause);

  factory MigrationError.checksumMismatch(
          String version, String expected, String actual) =>
      MigrationError(
          'Checksum mismatch for version $version: expected $expected, actual $actual',
          code: 'CHECKSUM_MISMATCH');

  factory MigrationError.directoryNotFound(String path) =>
      MigrationError('Directory not found: $path',
          code: 'DIRECTORY_NOT_FOUND');

  @override
  String toString() => 'MigrationError($code): $message';
}
