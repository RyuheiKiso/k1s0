import 'package:freezed_annotation/freezed_annotation.dart';

part 'claims.freezed.dart';
part 'claims.g.dart';

/// JWT Claims
///
/// Corresponds to the k1s0-auth crate Claims structure on the backend.
@freezed
class Claims with _$Claims {
  /// Creates JWT claims
  const factory Claims({
    /// Subject (user ID)
    required String sub,

    /// Issuer
    required String iss,

    /// Audience (may be string or list)
    @_AudienceConverter() List<String>? aud,

    /// Expiration time (Unix timestamp in seconds)
    required int exp,

    /// Issued at (Unix timestamp in seconds)
    required int iat,

    /// Not before (Unix timestamp in seconds)
    int? nbf,

    /// JWT ID
    String? jti,

    /// User roles
    @Default([]) List<String> roles,

    /// User permissions
    @Default([]) List<String> permissions,

    /// Tenant ID
    @JsonKey(name: 'tenant_id') String? tenantId,

    /// Scope
    String? scope,
  }) = _Claims;

  const Claims._();

  /// Creates claims from JSON
  factory Claims.fromJson(Map<String, dynamic> json) => _$ClaimsFromJson(json);

  /// Check if the token is expired
  bool get isExpired {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    return now >= exp;
  }

  /// Check if the token will expire within the given duration
  bool willExpireIn(Duration duration) {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final threshold = now + duration.inSeconds;
    return threshold >= exp;
  }

  /// Get the expiration time as DateTime
  DateTime get expirationTime =>
      DateTime.fromMillisecondsSinceEpoch(exp * 1000);

  /// Get the issued at time as DateTime
  DateTime get issuedAtTime => DateTime.fromMillisecondsSinceEpoch(iat * 1000);

  /// Check if the user has a specific role
  bool hasRole(String role) => roles.contains(role);

  /// Check if the user has any of the specified roles
  bool hasAnyRole(List<String> roleList) =>
      roleList.any((role) => roles.contains(role));

  /// Check if the user has all of the specified roles
  bool hasAllRoles(List<String> roleList) =>
      roleList.every((role) => roles.contains(role));

  /// Check if the user has a specific permission
  bool hasPermission(String permission) => permissions.contains(permission);

  /// Check if the user has any of the specified permissions
  bool hasAnyPermission(List<String> permissionList) =>
      permissionList.any((p) => permissions.contains(p));

  /// Check if the user has all of the specified permissions
  bool hasAllPermissions(List<String> permissionList) =>
      permissionList.every((p) => permissions.contains(p));
}

/// Converter for audience field that can be string or list
class _AudienceConverter implements JsonConverter<List<String>?, Object?> {
  const _AudienceConverter();

  @override
  List<String>? fromJson(Object? json) {
    if (json == null) return null;
    if (json is String) return [json];
    if (json is List) return json.map((e) => e.toString()).toList();
    return null;
  }

  @override
  Object? toJson(List<String>? object) {
    if (object == null) return null;
    if (object.length == 1) return object.first;
    return object;
  }
}
