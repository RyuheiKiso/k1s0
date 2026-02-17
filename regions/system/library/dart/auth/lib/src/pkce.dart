/// PKCE (Proof Key for Code Exchange) ユーティリティ
/// RFC 7636 準拠
library;

import 'dart:convert';
import 'dart:math';

import 'package:crypto/crypto.dart';

/// ランダムな code_verifier を生成する。
/// RFC 7636 Section 4.1 準拠。
/// [random] パラメータはテスト用に注入可能。
String generateCodeVerifier({Random? random}) {
  final rng = random ?? Random.secure();
  const chars =
      'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  return List.generate(43, (_) => chars[rng.nextInt(chars.length)]).join();
}

/// code_verifier から code_challenge を計算する（S256）。
/// SHA-256 ハッシュの Base64url エンコード（パディングなし）。
String generateCodeChallenge(String codeVerifier) {
  final bytes = utf8.encode(codeVerifier);
  final digest = sha256.convert(bytes);
  return base64UrlEncode(digest.bytes);
}

/// バイト列を Base64url エンコードする（パディングなし）。
/// RFC 4648 Section 5 準拠。
String base64UrlEncode(List<int> bytes) {
  return base64Url.encode(bytes).replaceAll('=', '');
}
