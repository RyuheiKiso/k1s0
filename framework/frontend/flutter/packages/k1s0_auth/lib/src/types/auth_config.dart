import 'package:freezed_annotation/freezed_annotation.dart';

part 'auth_config.freezed.dart';
part 'auth_config.g.dart';

/// Authentication configuration
@freezed
class AuthConfig with _$AuthConfig {
  /// Creates an authentication configuration
  const factory AuthConfig({
    /// Whether authentication is enabled
    @Default(true) bool enabled,

    /// Token storage type
    @Default(TokenStorageType.secure) TokenStorageType storageType,

    /// Time before expiration to trigger refresh (in seconds)
    @Default(300) int refreshMarginSeconds,

    /// Whether to automatically refresh tokens
    @Default(true) bool autoRefresh,

    /// Allowed issuers for token validation
    List<String>? allowedIssuers,

    /// Allowed audiences for token validation
    List<String>? allowedAudiences,

    /// OIDC configuration
    OidcConfig? oidc,
  }) = _AuthConfig;

  /// Creates an authentication configuration from JSON
  factory AuthConfig.fromJson(Map<String, dynamic> json) =>
      _$AuthConfigFromJson(json);
}

/// Token storage type
enum TokenStorageType {
  /// Secure storage (flutter_secure_storage)
  secure,

  /// In-memory storage (lost on app close)
  memory,
}

/// OIDC configuration
@freezed
class OidcConfig with _$OidcConfig {
  /// Creates an OIDC configuration
  const factory OidcConfig({
    /// Issuer URL
    required String issuer,

    /// Client ID
    required String clientId,

    /// Redirect URI
    required String redirectUri,

    /// Scopes
    @Default(['openid', 'profile', 'email']) List<String> scopes,

    /// Post-logout redirect URI
    String? postLogoutRedirectUri,

    /// Discovery URL (usually issuer + /.well-known/openid-configuration)
    String? discoveryUrl,
  }) = _OidcConfig;

  const OidcConfig._();

  /// Creates an OIDC configuration from JSON
  factory OidcConfig.fromJson(Map<String, dynamic> json) =>
      _$OidcConfigFromJson(json);

  /// Get the discovery URL
  String get effectiveDiscoveryUrl =>
      discoveryUrl ?? '$issuer/.well-known/openid-configuration';
}
