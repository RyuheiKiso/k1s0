import 'package:jwt_decoder/jwt_decoder.dart';

import 'claims.dart';

/// Token decoding error
class TokenDecodingError implements Exception {
  /// Creates a token decoding error
  TokenDecodingError(this.message, [this.cause]);

  /// Error message
  final String message;

  /// Original exception
  final Object? cause;

  @override
  String toString() => 'TokenDecodingError: $message';
}

/// Decode a JWT token and extract claims
Claims decodeJwtToken(String token) {
  try {
    final payload = JwtDecoder.decode(token);
    return Claims.fromJson(payload);
  } on FormatException catch (e) {
    throw TokenDecodingError('Failed to decode JWT token', e);
  } on Exception catch (e) {
    throw TokenDecodingError('Failed to decode JWT token', e);
  }
}

/// Try to decode a JWT token, returning null on failure
Claims? tryDecodeJwtToken(String token) {
  try {
    return decodeJwtToken(token);
  } on TokenDecodingError {
    return null;
  }
}

/// Check if a token is expired
bool isJwtTokenExpired(String token) {
  try {
    return JwtDecoder.isExpired(token);
  } on FormatException {
    return true;
  } on Exception {
    return true;
  }
}

/// Get the expiration date of a token
DateTime? getJwtExpirationDate(String token) {
  try {
    return JwtDecoder.getExpirationDate(token);
  } on FormatException {
    return null;
  } on Exception {
    return null;
  }
}

/// Get remaining time until token expires
Duration? getJwtRemainingTime(String token) {
  try {
    return JwtDecoder.getRemainingTime(token);
  } on FormatException {
    return null;
  } on Exception {
    return null;
  }
}

/// Validate a token structure (does not verify signature)
bool isValidJwtFormat(String token) {
  try {
    // JWT should have 3 parts separated by dots
    final parts = token.split('.');
    if (parts.length != 3) return false;

    // Try to decode to verify structure
    JwtDecoder.decode(token);
    return true;
  } on FormatException {
    return false;
  } on Exception {
    return false;
  }
}

/// JWT token decoder
///
/// This class provides backward compatibility.
/// Consider using the top-level functions instead.
class TokenDecoder {
  TokenDecoder._();

  /// Decode a JWT token and extract claims
  static Claims decode(String token) => decodeJwtToken(token);

  /// Try to decode a JWT token, returning null on failure
  static Claims? tryDecode(String token) => tryDecodeJwtToken(token);

  /// Check if a token is expired
  static bool isExpired(String token) => isJwtTokenExpired(token);

  /// Get the expiration date of a token
  static DateTime? getExpirationDate(String token) =>
      getJwtExpirationDate(token);

  /// Get remaining time until token expires
  static Duration? getRemainingTime(String token) => getJwtRemainingTime(token);

  /// Validate a token structure (does not verify signature)
  static bool isValidFormat(String token) => isValidJwtFormat(token);
}
