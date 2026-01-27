import 'package:freezed_annotation/freezed_annotation.dart';

import '../token/claims.dart';

part 'auth_user.freezed.dart';

/// Authenticated user information
@freezed
class AuthUser with _$AuthUser {
  /// Creates an authenticated user
  const factory AuthUser({
    /// User ID
    required String id,

    /// User roles
    required List<String> roles,

    /// User permissions
    required List<String> permissions,

    /// Tenant ID
    String? tenantId,

    /// JWT claims
    required Claims claims,
  }) = _AuthUser;

  const AuthUser._();

  /// Create an AuthUser from Claims
  factory AuthUser.fromClaims(Claims claims) {
    return AuthUser(
      id: claims.sub,
      roles: claims.roles,
      permissions: claims.permissions,
      tenantId: claims.tenantId,
      claims: claims,
    );
  }

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

  /// Check if the user is an admin
  bool get isAdmin => hasRole('admin') || hasRole('administrator');
}
