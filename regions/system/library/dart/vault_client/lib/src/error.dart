enum VaultErrorCode {
  notFound,
  permissionDenied,
  serverError,
  timeout,
  leaseExpired,
}

class VaultError implements Exception {
  final VaultErrorCode code;
  final String message;

  const VaultError(this.code, this.message);

  @override
  String toString() => 'VaultError(${code.name}): $message';
}
