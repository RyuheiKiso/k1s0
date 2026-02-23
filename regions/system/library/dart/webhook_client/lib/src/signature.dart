import 'dart:convert';

import 'package:crypto/crypto.dart';

String generateSignature(String secret, String body) {
  final key = utf8.encode(secret);
  final message = utf8.encode(body);
  final hmac = Hmac(sha256, key);
  return hmac.convert(message).toString();
}

bool verifySignature(String secret, String body, String signature) =>
    generateSignature(secret, body) == signature;
