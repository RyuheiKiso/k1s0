import 'error.dart';

final _emailRegExp = RegExp(r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$');
final _uuidRegExp = RegExp(
  r'^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$',
  caseSensitive: false,
);
final _tenantIdRegExp = RegExp(r'^[a-z0-9-]{3,63}$');

void validateEmail(String email) {
  if (!_emailRegExp.hasMatch(email)) {
    throw ValidationError('email', 'invalid email format: $email', code: 'INVALID_EMAIL');
  }
}

void validateUuid(String id) {
  if (!_uuidRegExp.hasMatch(id)) {
    throw ValidationError('id', 'invalid UUID v4 format: $id', code: 'INVALID_UUID');
  }
}

void validateUrl(String url) {
  final uri = Uri.tryParse(url);
  if (uri == null || !uri.hasScheme || (uri.scheme != 'http' && uri.scheme != 'https')) {
    throw ValidationError('url', 'invalid URL: $url', code: 'INVALID_URL');
  }
}

void validateTenantId(String tenantId) {
  if (!_tenantIdRegExp.hasMatch(tenantId)) {
    throw ValidationError('tenantId', 'invalid tenant ID: $tenantId', code: 'INVALID_TENANT_ID');
  }
}

void validatePagination(int page, int perPage) {
  if (page < 1) {
    throw ValidationError('page', 'page must be >= 1, got $page', code: 'INVALID_PAGE');
  }
  if (perPage < 1 || perPage > 100) {
    throw ValidationError('perPage', 'perPage must be 1-100, got $perPage', code: 'INVALID_PER_PAGE');
  }
}

void validateDateRange(DateTime startDate, DateTime endDate) {
  if (startDate.isAfter(endDate)) {
    throw ValidationError(
      'dateRange',
      'start date ($startDate) must be <= end date ($endDate)',
      code: 'INVALID_DATE_RANGE',
    );
  }
}
