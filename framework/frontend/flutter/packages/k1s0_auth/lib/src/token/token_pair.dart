import 'package:freezed_annotation/freezed_annotation.dart';

part 'token_pair.freezed.dart';
part 'token_pair.g.dart';

/// Token pair containing access and refresh tokens
@freezed
class TokenPair with _$TokenPair {
  /// Creates a token pair
  const factory TokenPair({
    /// Access token (JWT)
    @JsonKey(name: 'access_token') required String accessToken,

    /// Refresh token
    @JsonKey(name: 'refresh_token') String? refreshToken,

    /// ID token (OIDC)
    @JsonKey(name: 'id_token') String? idToken,

    /// Access token expiration time (Unix timestamp in milliseconds)
    @JsonKey(name: 'expires_at') int? expiresAt,

    /// Token type (usually "Bearer")
    @JsonKey(name: 'token_type') @Default('Bearer') String tokenType,

    /// Scopes
    String? scope,
  }) = _TokenPair;

  const TokenPair._();

  /// Creates a token pair from JSON
  factory TokenPair.fromJson(Map<String, dynamic> json) =>
      _$TokenPairFromJson(json);

  /// Check if the token is expired based on expiresAt
  bool get isExpired {
    if (expiresAt == null) return false;
    return DateTime.now().millisecondsSinceEpoch >= expiresAt!;
  }

  /// Check if the token will expire within the given duration
  bool willExpireIn(Duration duration) {
    if (expiresAt == null) return false;
    final threshold =
        DateTime.now().millisecondsSinceEpoch + duration.inMilliseconds;
    return threshold >= expiresAt!;
  }

  /// Get the expiration time as DateTime
  DateTime? get expirationTime =>
      expiresAt != null ? DateTime.fromMillisecondsSinceEpoch(expiresAt!) : null;

  /// Check if this token pair has a refresh token
  bool get hasRefreshToken => refreshToken != null && refreshToken!.isNotEmpty;

  /// Create the Authorization header value
  String get authorizationHeader => '$tokenType $accessToken';
}
