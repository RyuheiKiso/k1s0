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

/// JWT token decoder
class TokenDecoder {
  /// Decode a JWT token and extract claims
  static Claims decode(String token) {
    try {
      final payload = JwtDecoder.decode(token);
      return Claims.fromJson(payload);
    } catch (e) {
      throw TokenDecodingError('Failed to decode JWT token', e);
    }
  }

  /// Try to decode a JWT token, returning null on failure
  static Claims? tryDecode(String token) {
    try {
      return decode(token);
    } catch (_) {
      return null;
    }
  }

  /// Check if a token is expired
  static bool isExpired(String token) {
    try {
      return JwtDecoder.isExpired(token);
    } catch (_) {
      return true;
    }
  }

  /// Get the expiration date of a token
  static DateTime? getExpirationDate(String token) {
    try {
      return JwtDecoder.getExpirationDate(token);
    } catch (_) {
      return null;
    }
  }

  /// Get remaining time until token expires
  static Duration? getRemainingTime(String token) {
    try {
      final exp = JwtDecoder.getRemainingTime(token);
      return exp;
    } catch (_) {
      return null;
    }
  }

  /// Validate a token structure (does not verify signature)
  static bool isValidFormat(String token) {
    try {
      // JWT should have 3 parts separated by dots
      final parts = token.split('.');
      if (parts.length != 3) return false;

      // Try to decode to verify structure
      JwtDecoder.decode(token);
      return true;
    } catch (_) {
      return false;
    }
  }
}
