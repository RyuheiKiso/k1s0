import 'error.dart';

final _emailRegExp = RegExp(r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$');
final _uuidRegExp = RegExp(
  r'^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$',
  caseSensitive: false,
);
final _tenantIdRegExp = RegExp(r'^[a-z0-9-]{3,63}$');

void validateEmail(String email) {
  if (!_emailRegExp.hasMatch(email)) {
    throw ValidationError('email', 'invalid email format: $email');
  }
}

void validateUuid(String id) {
  if (!_uuidRegExp.hasMatch(id)) {
    throw ValidationError('id', 'invalid UUID v4 format: $id');
  }
}

void validateUrl(String url) {
  final uri = Uri.tryParse(url);
  if (uri == null || !uri.hasScheme || (uri.scheme != 'http' && uri.scheme != 'https')) {
    throw ValidationError('url', 'invalid URL: $url');
  }
}

void validateTenantId(String tenantId) {
  if (!_tenantIdRegExp.hasMatch(tenantId)) {
    throw ValidationError('tenantId', 'invalid tenant ID: $tenantId');
  }
}
