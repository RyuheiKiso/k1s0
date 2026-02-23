enum TenantErrorCode { notFound, suspended, serverError, timeout }

class TenantError implements Exception {
  final String message;
  final TenantErrorCode code;

  const TenantError(this.message, this.code);

  @override
  String toString() => 'TenantError($code): $message';
}
